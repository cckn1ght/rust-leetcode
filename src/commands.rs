use clap::ArgMatches;
use crate::helper::*;

pub fn run_setup_command(matches: &ArgMatches) {
    let path = create_project_folder(matches).unwrap_or_else(handle_error);
    create_templates(path).unwrap_or_else(handle_error);
}
pub fn run_random_command(matches: &ArgMatches) {

    let difficulty = matches.value_of("difficulty");
    let difficulty_num: Option<u32> = difficulty.map(|d| match d.to_lowercase().as_str() {
        "easy" => 1,
        "medium" => 2,
        "hard" => 3,
        num@ "1" | num@ "2" | num @ "3" =>
            num.parse::<u32>().unwrap(),
        _ => 0,
    });

    start_solve(get_one_random_problem(difficulty_num));
}
pub fn run_solve_command(matches: &ArgMatches) {
    let problem_id = match matches.value_of("INPUT").unwrap().parse::<u32>() {
        Ok(id) => id,
        Err(_) => panic!("please input valid leetcode question id!"),
    };
    start_solve(get_one_problem(problem_id));
}
