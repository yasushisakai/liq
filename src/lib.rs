use bs58::encode;
use ndarray::{s, stack, Array, Array2, Axis};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashSet, BTreeMap};
use std::iter::FromIterator;

#[derive(Debug, Deserialize, Serialize)]
pub struct Plan {
    title: String,
    description: Option<String>,
}

impl Plan {
    pub fn new(title: String) -> Self {
        Plan {
            title,
            description: None,
        }
    }
}

impl PartialEq for Plan {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
    }
}

impl Eq for Plan {}

// the design princicple of this struct is that it is human understandable,
// and easy to edit. Editing a raw matrix will not be as straight forward
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Setting {
    voters: Vec<String>,
    plans: Vec<Plan>,
    votes: BTreeMap<String, BTreeMap<String, f64>>,
}

impl Setting {
    pub fn new() -> Self {
        Setting {
            voters: Vec::new(),
            plans: Vec::new(),
            votes: BTreeMap::new(),
        }
    }

    pub fn based_hash(&self) -> String {
        let votes = serde_json::to_vec(&self.votes).unwrap();
        encode(Sha256::digest(&votes)).into_string()
    }

    pub fn add_voter(&mut self, p: &str) {
        if !self.voters.iter().any(|u| u == p) {
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

    pub fn add_plan(&mut self, plan: Plan) {
        if !self.plans.iter().any(|p| p == &plan) {
            self.plans.push(plan);
        }
    }

    pub fn get_voters(&self) -> HashSet<String> {
        HashSet::from_iter(self.voters.iter().map(|v|v.to_string())) 
    } 

    pub fn delete_plan(&mut self, other_title: &String) -> Option<usize> {
        match self.plans.iter().position(|p| &p.title == other_title) {
            Some(index) => {
                self.plans.remove(index);
                Some(index)
            }
            None => None,
        }
    }

    pub fn cast_vote(&mut self, voter: &str, plan_or_voter: &str, value: f64) {
        if !self.voters.iter().any(|v| v == voter) {
            return;
        }

        if !self.plans.iter().any(|p| p.title == plan_or_voter)
            && !self.voters.iter().any(|v| v == plan_or_voter)
        {
            return;
        }

        match self.votes.keys().any(|v| v == voter) {
            true => {
                self.votes
                    .get_mut(voter)
                    .and_then(|vote| vote.insert(plan_or_voter.to_string(), value));
            }
            false => {
                let mut vote: BTreeMap<String, f64> = BTreeMap::new();
                vote.insert(plan_or_voter.to_string(), value);
                self.votes.insert(voter.to_string(), vote);
            }
        }
    }

    pub fn overwrite_vote(&mut self, voter: &str, vote: BTreeMap<String, f64>) {
        if self.voters.iter().any(|v| v == voter) {
            self.votes.insert(voter.into(), vote);
        }
    }

    pub fn purge_and_normalize(&mut self) {
        let new_votes: BTreeMap<String, BTreeMap<String, f64>> = self
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

                let nv: BTreeMap<String, f64> = votes
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

    pub fn calculate(&self) -> PollResult {
        let mat = create_matrix(self);
        let raw_result = calculate(mat, self.voters.len());
        poll_result(&self.voters, &self.plans, raw_result)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PollResult {
    votes: BTreeMap<String, Option<f64>>,
    influence: BTreeMap<String, Option<f64>>,
}

impl PollResult {
    pub fn based_hash(&self) -> String {
        serde_json::to_vec(self)
            .and_then(|v| Ok(Sha256::digest(&v)))
            .and_then(|h| Ok(encode(h).into_string()))
            .expect("Poll Result should be able to Serialize")
    }
}

const ITERATION: u32 = 1000;

fn create_matrix(settings: &Setting) -> Array2<f64> {
    // There will be one more policy added for calculation.
    // This is the default policy where voters vote placed by default

    let voters = &settings.voters;
    // TODO: check for duplicates
    let plans = &settings.plans;

    let p_num = plans.len() + 1;

    let elements_num = voters.len() + p_num;

    let mut m = Array::zeros((elements_num, voters.len()));

    for (i, v) in voters.iter().enumerate() {
        match settings.votes.get(v) {
            Some(vote) => {
                for (key, val) in vote.iter() {
                    let id = match voters.iter().position(|k| k == key) {
                        Some(n) => Some(n),
                        None => match &plans.iter().position(|k| &k.title == key) {
                            Some(n) => Some(n + voters.len()),
                            None => {
                                println!("W: {} was not found in voters nor policies!", &key);
                                None
                            }
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

    let vp: Array2<f64> = Array::zeros((plans.len() + 1, voters.len()));
    let pp: Array2<f64> = Array::eye(plans.len() + 1);

    let leftpart = stack![Axis(1), vp, pp];
    let initial_matrix = stack![Axis(1), m, leftpart.t()];

    initial_matrix
}

fn calculate(m: Array2<f64>, num_voters: usize) -> (Vec<f64>, Vec<f64>) {
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

fn poll_result(voters: &[String], plans: &[Plan], result: (Vec<f64>, Vec<f64>)) -> PollResult {
    let mut votes_r = BTreeMap::new();

    let mut influences_r = BTreeMap::new();

    let (votes, influence) = result;

    for (i, p) in plans.iter().enumerate() {
        votes_r.insert(p.title.to_owned(), Some(votes.get(i).unwrap().to_owned()));
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
