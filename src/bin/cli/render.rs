use std::error::Error;
use std::iter::FromIterator;
use std::cmp::Ordering;
use liquid_demons::{Setting, PollResult};
use serde_json::{Map, Value};
use std::io::prelude::*;
use term;

pub fn pretty_print_settings(setting: &Setting) -> Result<(), Box<dyn Error>> {
    let mut t = term::stdout().unwrap();
    let mut max_length = 0;
    for v in &setting.voters {
        if max_length < v.len() {
            max_length = v.len();
        }
    }

    for p in &setting.policies {
        if max_length < p.len() {
            max_length = p.len();
        }
    }

    max_length = max_length;

    t.attr(term::Attr::Bold)?;
    write!(t, "{:10}", "title")?;
    t.reset()?;

    if let Some(title) = &setting.title {
        write!(t, "{}\n", title)?;
    }

    t.attr(term::Attr::Bold)?;
    write!(t, "{:10}", "policies")?;
    t.reset()?;

    t.fg(term::color::GREEN)?; 
    for p in &setting.policies {
        write!(t, "{:width$} ", p, width=&max_length)?;
    }

    t.reset()?;
    write!(t, "\n")?;

    t.attr(term::Attr::Bold)?;
    write!(t, "{:10}","voters")?;
    t.reset()?;

    t.fg(term::color::BLUE)?; 
    for v in &setting.voters {
        write!(t, "{:width$} ", v, width=&max_length)?;
    }
    t.reset()?;

    write!(t, "\n\n")?;

    let votes: &Map<String, Value> = setting.votes.as_object().unwrap();

    t.attr(term::Attr::Bold)?;
    write!(t, "{:10}","votes")?;
    t.reset()?;

    println!(); 

    for (from, vote_value) in Vec::from_iter(votes.iter()) {
        t.fg(term::color::BLUE)?;
        t.attr(term::Attr::Bold)?;
        write!(t, "  {:width$}", from, width=&max_length)?;
        t.reset()?;
        write!(t, " → ")?;
        let vote: &Map<String, Value> = vote_value.as_object().unwrap();

        for policy in &setting.policies {
            if let Some(value) = vote.get(policy) {
                t.fg(term::color::GREEN)?;
                write!(t, "{:width$}: ", policy, width=&max_length)?;
                t.reset()?;
                write!(t, "{:width$}, ", value, width=&max_length)?;
            }
        }

        for voter in &setting.voters {
            if let Some(value) = vote.get(voter) {
                t.fg(term::color::BLUE)?;
                write!(t, "{:width$}: ", voter, width=&max_length)?;
                t.reset()?;
                write!(t, "{:width$}, ", value, width=&max_length)?;
            }
        }

        write!(t, "\n")?;
    }

    write!(t, "\n")?;
    Ok(())
}

pub fn pretty_print_result(result: &PollResult) -> Result<(), Box<dyn Error>> {

    let mut t = term::stdout().unwrap();

    // sort the results
    let mut votes = Vec::from_iter(&result.votes);
    votes.sort_by(|&(_, a), &(_, b)| b.partial_cmp(&a).unwrap_or(Ordering::Equal));
    let mut influence = Vec::from_iter(&result.influence);
    influence.sort_by(|&(_, a), &(_, b)| b.partial_cmp(&a).unwrap_or(Ordering::Equal));
    let mut max_length = 0;

    for (v, _) in &votes {
        if max_length < v.len()  {
            max_length = v.len()
        }   
    } 

    for (i, _) in &influence {
        if max_length < i.len() {
            max_length = i.len()
        }
    }

    t.attr(term::Attr::Bold)?;
    write!(t, "result\n  policies:\n")?;
    t.reset()?;

    t.fg(term::color::GREEN)?;
    t.attr(term::Attr::Bold)?;
    // t.fg(term::color::WHITE).unwrap();
    t.reset()?;
    for (p, v) in &votes {
        t.attr(term::Attr::Bold)?;
        t.fg(term::color::GREEN)?;
        write!(t, "  {:width$} ", p, width=&max_length)?;
        t.reset()?;
        write!(t, "{:.4}\n", v)?;
    }

    write!(t, "\r\n")?;

    t.attr(term::Attr::Bold)?;
    write!(t, "  influence:\n")?;
    t.reset()?;

    t.fg(term::color::BLUE)?;
    t.attr(term::Attr::Bold)?;
    t.reset()?;
    for (inf, v) in &influence {
        t.attr(term::Attr::Bold)?;
        t.fg(term::color::BLUE)?;
        write!(t, "  {:width$} ", inf, width=&max_length)?;
        t.reset()?;
        write!(t, "{:.4}\n", v)?;
    }
    Ok(())
}