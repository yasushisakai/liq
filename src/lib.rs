use ndarray::{s, stack, Array, Array2, Axis};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use sha2::{Sha256, Digest};
use bs58::encode;

#[derive(Debug, Deserialize, Serialize)]
pub enum Policy {
    Short(String),
    Long((String, String)),
}

impl fmt::Display for Policy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = match self {
            Policy::Short(d) => d,
            Policy::Long((d, _)) => d,
        };
        write!(f, "{}", d)
    }
}

pub fn match_by_string(policy: &Policy, id: &str) -> bool {
    match policy {
        Policy::Short(desc) => desc == id,
        Policy::Long((title, _)) => title == id,
    }
}

// the design princicple of this struct is that it is human understandable,
// and easy to edit. Editing a raw matrix will not be as straight forward
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Setting {
    pub id: String,
    pub title: Option<String>,
    pub voters: Vec<String>,
    pub policies: Vec<Policy>,
    pub prev_id: String,
    pub votes: HashMap<String, HashMap<String, f64>>,
}

impl Setting {
    pub fn new() -> Self {
        Setting {
            id: String::new(),
            prev_id: String::new(),
            title: None,
            voters: Vec::new(),
            policies: Vec::new(),
            votes: HashMap::new(),
        }
    }

    pub fn compute_id(&self) -> String {
        let votes = serde_json::to_vec(&self.votes).unwrap();
        encode(Sha256::digest(&votes)).into_string()
    }

    pub fn add_voter(&mut self, p: &str) {
        if !self.voters.iter().any(|u|u==p){
            self.voters.push(p.to_string());
        }
    }

    pub fn delete_voter(&mut self, p: &str) -> Option<usize> {
        match self.voters.iter().position(|v| v == p) {
            Some(index) => {
                self.voters.remove(index);
                let _ = self.votes.remove(p);
                Some(index)
            }
            None => None,
        }
    }

    pub fn add_policy(&mut self, policy: Policy) {
        if !self.policies.iter().any(|p|p.to_string()==policy.to_string()){
            self.policies.push(policy);
        }
    }

    pub fn delete_policy(&mut self, p: &str) -> Option<usize> {
        match self.policies.iter().position(|v| match_by_string(v, p)) {
            Some(index) => {
                self.policies.remove(index);
                Some(index)
            }
            None => None,
        }
    }

    pub fn purge_and_normalize(&mut self) {
        let new_votes: HashMap<String, HashMap<String, f64>> = self
            .votes
            .iter()
            .filter_map(|(voter, votes)| {
                if votes.is_empty() {
                    return None;
                };

                let sum = votes.iter().by_ref().fold(0.0, |s, (_, v)| s + v);

                if sum == 0.0f64 {
                    return None;
                };

                let nv: HashMap<String, f64> = votes
                    .iter()
                    .filter_map(move |(to, vote)| {
                        if *vote == 0.0 {
                            None
                        } else {
                            Some((to.to_string(), *vote / sum))
                        }
                    })
                    .collect();

                Some((voter.to_string(), nv))
            })
            .collect();
        self.votes = new_votes;
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PollResult {
    pub votes: HashMap<String, Option<f64>>,
    pub influence: HashMap<String, Option<f64>>,
}

const ITERATION: u32 = 1000;

pub fn create_matrix(settings: &Setting) -> Array2<f64> {
    // There will be one more policy added for calculation.
    // This is the default policy where voters vote placed by default

    let voters = &settings.voters;
    // TODO: check for duplicates
    let policies = &settings.policies;

    let p_num = policies.len() + 1;

    let elements_num = voters.len() + p_num;

    let mut m = Array::zeros((elements_num, voters.len()));

    for (i, v) in voters.iter().enumerate() {
        match settings.votes.get(v) {
            Some(vote) => {
                for (key, val) in vote.iter() {
                    let id = match voters.iter().position(|k| k == key) {
                        Some(n) => Some(n),
                        None => match &policies.iter().position(|k| match_by_string(k, key)) {
                            Some(n) => Some(n + voters.len()),
                            None => {
                                println!("W: {} was not found in voters nor policies!", &key);
                                None
                            },
                        },
                    };

                    if let Some(index) = id {
                        m[[index, i]] = val.to_owned();
                    }
                }

            }
            None => {
                m[[elements_num - 1, i]] = 1.0;
            }
        }
    }

    let sum_row = m.sum_axis(Axis(0));

    // normalize
    for f in 0..voters.len() {
        for t in 0..elements_num {
            m[[t, f]] /= sum_row[[f]];
        }
    }

    let vp: Array2<f64> = Array::zeros((policies.len() + 1, voters.len()));
    let pp: Array2<f64> = Array::eye(policies.len() + 1);

    let leftpart = stack![Axis(1), vp, pp];
    let initial_matrix = stack![Axis(1), m, leftpart.t()];

    initial_matrix
}

pub fn calculate(m: Array2<f64>, num_voters: usize) -> (Vec<f64>, Vec<f64>) {

    let square = m.shape()[0];
    let mut a = Array::eye(square);
    let mut sum = Array::eye(square);

    for _i in 0..ITERATION {
        a = a.dot(&m);
        sum += &a;
    }

    let a = a.slice(s![.., 0..num_voters]); 
    let vote_results = a.sum_axis(Axis(1));
    let vote_results = vote_results.slice(s![num_voters..]).to_vec();

    let sum = sum.slice(s![..num_voters, ..num_voters]);
    let sum_row = sum.sum_axis(Axis(1));
    let voters_influence = (sum_row / sum.diag()).to_vec();

    (vote_results, voters_influence)
}

pub fn poll_result(
    voters: &[String],
    policies: &[Policy],
    result: (Vec<f64>, Vec<f64>),
) -> PollResult {
    let mut votes_r = HashMap::new();

    let mut influences_r = HashMap::new();

    let (votes, influence) = result;

    for (i, p) in policies.iter().enumerate() {
        let d = format!("{}", p);
        votes_r.insert(d.to_owned(), Some(votes.get(i).unwrap().to_owned()));
    }

    votes_r.insert("(Blank)".to_owned(), Some(votes.last().unwrap().to_owned()));

    for (i, inf) in voters.iter().enumerate() {
        influences_r.insert(inf.to_owned(), Some(influence.get(i).unwrap().to_owned()));
    }

    PollResult {
        votes: votes_r,
        influence: influences_r,
    }
}
