

use crate::fetcher;

use regex::Regex;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;

use crate::fetcher::{Problem, Problems, CodeDefinition};
use std::fmt::Display;
use clap::ArgMatches;
use std::process::Command;
use std::collections::HashSet;

pub fn start_solve(problem: Problem) {
    match get_code_of_problem(&problem) {
        None => println!("Problem {} has no rust version.", problem.question_id),
        Some(code) => {
            let solution_name = get_solution_name(&problem);
            let solutions_path = env::current_dir().unwrap().join("src").join("solutions");
            let mut mod_file = fs::OpenOptions::new()
                .append(true)
                .open(solutions_path.join("mod.rs"))
                .unwrap();
            writeln!(mod_file, "mod {};", solution_name).unwrap();

            let solution_path = solutions_path.join(format!("{}.rs", solution_name));
            if solution_path.exists() {
                panic!("solution exists");
            }
            let mut file = File::create(&solution_path).unwrap();
            file.write_all(solution_template(&problem, code).as_bytes()).unwrap();
        }
    }
}

pub fn get_one_random_problem(difficulty_num: Option<u32>) -> Problem {
    use rand::Rng;
    let problems = get_problems_from_file();
    let exists_ids = exists_problem_ids();
    let status_filter = problems.stat_status_pairs
        .iter()
        .filter(|&stat| !exists_ids.contains(&stat.stat.frontend_question_id));
    let status = match difficulty_num {
        Some(diffi) => status_filter
            .filter(|&stat| stat.difficulty.level == diffi)
            .collect::<Vec<_>>(),
        None => status_filter.collect::<Vec<_>>(),
    };
    let mut rng = rand::thread_rng();
    let rand_id: usize = rng.gen_range(0, status.len());
    let stat = status[rand_id];
    fetcher::get_problem_by_stat(stat).unwrap()
}
pub fn get_one_problem(frontend_question_id: u32) -> Problem {
    let ps = get_problems_from_file();
    let stat = ps.stat_status_pairs
        .iter()
        .find(|&st| st.stat.frontend_question_id == frontend_question_id);
    match stat {
        Some(stat) => fetcher::get_problem_by_stat(stat).unwrap(),
        None => panic!(format!("can not find leetcode question of {}", frontend_question_id)),
    }
}

pub fn get_problems_from_file() -> Problems {
    let path = env::current_dir()
        .unwrap_or_else(handle_error)
        .join("src")
        .join(".problems");
    let file = match fs::File::open(path) {
        Err(_) => panic!("make sure you are in the correct rust-leetcode project folder"),
        Ok(file) => file,
    };
    let reader = io::BufReader::new(file);
    serde_json::from_reader(reader).expect("seems like the src/.problems file has been changed!")
}

pub fn fill_problem_list(file: fs::File, filter_out_paid: bool) {
    let mut problems = fetcher::get_problems().unwrap();
    if filter_out_paid {
        problems.stat_status_pairs = problems.stat_status_pairs
            .into_iter()
            .filter(|s| !s.paid_only)
            .collect();
    }
    let file= io::BufWriter::new(file);
    serde_json::to_writer(file, &problems).unwrap_or_else(handle_error)
}

pub fn create_templates(path: PathBuf) -> io::Result<()> {
    let main_path = path.join("src").join("main.rs");
    let main_content = include_bytes!("templates/main.template");
    let mut main_file = fs::File::create(main_path)?;
    main_file.write_all(main_content)?;
    let solutions_path = path.join("src").join("solutions");
    fs::create_dir(solutions_path.clone())?;
    let src_mod_path = solutions_path.join("mod.rs");
    fs::File::create(src_mod_path)?;
    let problems_path = path.join("src").join(".problems");
    let problem_list_file = fs::File::create(problems_path)?;
    fill_problem_list(problem_list_file, true);
    Ok(())
}

pub fn create_project_folder(matches: &ArgMatches) -> io::Result<PathBuf> {
    let path = project_path(matches);
    let cargo_command = Command::new("cargo")
        .arg("new")
        .arg(path.clone().to_str().unwrap())
        .output().expect("failed to execute cargo");
    io::stdout().write_all(&cargo_command.stdout)?;
    io::stderr().write_all(&cargo_command.stderr)?;
    Ok(path)
}

pub fn project_path(matches: &ArgMatches) -> PathBuf {
    let project_name = matches
        .value_of("PROJECT_NAME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("RL_PROJECT_NAME").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("leetcode-rust"));
    env::current_dir().unwrap_or_else(handle_error).join(project_name)
}

#[allow(clippy::needless_pass_by_value)]
pub fn handle_error<E: Display, T>(error: E) -> T {
    eprintln!("{}", error);
    ::std::process::exit(1);
}

fn get_code_of_problem(problem: &Problem) -> Option<&CodeDefinition> {
    problem
        .code_definition
        .iter()
        .find(|&d| d.value == "rust".to_string())
}
fn get_solution_name(problem: &Problem) -> String {
    format!(
        "{}_{:04}_{}",
        problem.difficulty.to_lowercase(),
        problem.question_id,
        problem.title_slug.replace("-", "_")
    )
}
fn solution_template(problem: &Problem, code: &CodeDefinition) -> String {
    let template = include_str!("templates/solution.template");
    template
        .replace("__PROBLEM_TITLE__", &problem.title)
        .replace("__PROBLEM_DESC__", &build_desc(&problem.content))
        .replace(
            "__PROBLEM_DEFAULT_CODE__",
            &insert_return_in_code(&problem.return_type, &code.default_code),
        )
        .replace("__PROBLEM_ID__", &format!("{}", problem.question_id))
        .replace("__EXTRA_USE__", &parse_extra_use(&code.default_code))
        .replace("__PROBLEM_LINK__", &parse_problem_link(&problem))
        .replace("__DISCUSS_LINK__", &parse_discuss_link(&problem))
}

fn parse_extra_use(code: &str) -> String {
    let mut extra_use_line = String::new();
    // a linked-list problem
    if code.contains("pub struct ListNode") {
        extra_use_line.push_str("\nuse crate::util::linked_list::{ListNode, to_list};")
    }
    if code.contains("pub struct TreeNode") {
        extra_use_line.push_str("\nuse crate::util::tree::{TreeNode, to_tree};")
    }
    if code.contains("pub struct Point") {
        extra_use_line.push_str("\nuse crate::util::point::Point;")
    }
    extra_use_line
}

fn parse_problem_link(problem: &fetcher::Problem) -> String {
    format!("https://leetcode.com/problems/{}/", problem.title_slug)
}

fn parse_discuss_link(problem: &fetcher::Problem) -> String {
    format!(
        "https://leetcode.com/problems/{}/discuss/?currentPage=1&orderBy=most_votes&query=",
        problem.title_slug
    )
}

fn insert_return_in_code(return_type: &str, code: &str) -> String {
    let re = Regex::new(r"\{[ \n]+}").unwrap();
    match return_type {
        "ListNode" => re
            .replace(&code, "{\n        Some(Box::new(ListNode::new(0)))\n    }")
            .to_string(),
        "ListNode[]" => re.replace(&code, "{\n        vec![]\n    }").to_string(),
        "TreeNode" => re
            .replace(
                &code,
                "{\n        Some(Rc::new(RefCell::new(TreeNode::new(0))))\n    }",
            )
            .to_string(),
        "boolean" => re.replace(&code, "{\n        false\n    }").to_string(),
        "character" => re.replace(&code, "{\n        '0'\n    }").to_string(),
        "character[][]" => re.replace(&code, "{\n        vec![]\n    }").to_string(),
        "double" => re.replace(&code, "{\n        0f64\n    }").to_string(),
        "double[]" => re.replace(&code, "{\n        vec![]\n    }").to_string(),
        "int[]" => re.replace(&code, "{\n        vec![]\n    }").to_string(),
        "integer" => re.replace(&code, "{\n        0\n    }").to_string(),
        "integer[]" => re.replace(&code, "{\n        vec![]\n    }").to_string(),
        "integer[][]" => re.replace(&code, "{\n        vec![]\n    }").to_string(),
        "list<String>" => re.replace(&code, "{\n        vec![]\n    }").to_string(),
        "list<TreeNode>" => re.replace(&code, "{\n        vec![]\n    }").to_string(),
        "list<boolean>" => re.replace(&code, "{\n        vec![]\n    }").to_string(),
        "list<double>" => re.replace(&code, "{\n        vec![]\n    }").to_string(),
        "list<integer>" => re.replace(&code, "{\n        vec![]\n    }").to_string(),
        "list<list<integer>>" => re.replace(&code, "{\n        vec![]\n    }").to_string(),
        "list<list<string>>" => re.replace(&code, "{\n        vec![]\n    }").to_string(),
        "list<string>" => re.replace(&code, "{\n        vec![]\n    }").to_string(),
        "null" => code.to_string(),
        "string" => re
            .replace(&code, "{\n        String::new()\n    }")
            .to_string(),
        "string[]" => re.replace(&code, "{\n        vec![]\n    }").to_string(),
        "void" => code.to_string(),
        "NestedInteger" => code.to_string(),
        "Node" => code.to_string(),
        _ => code.to_string(),
    }
}

fn build_desc(content: &str) -> String {
    // TODO: fix this shit
    content
        .replace("<strong>", "")
        .replace("</strong>", "")
        .replace("<em>", "")
        .replace("</em>", "")
        .replace("</p>", "")
        .replace("<p>", "")
        .replace("<b>", "")
        .replace("</b>", "")
        .replace("<pre>", "")
        .replace("</pre>", "")
        .replace("<ul>", "")
        .replace("</ul>", "")
        .replace("<li>", "")
        .replace("</li>", "")
        .replace("<code>", "")
        .replace("</code>", "")
        .replace("<i>", "")
        .replace("</i>", "")
        .replace("<sub>", "")
        .replace("</sub>", "")
        .replace("</sup>", "")
        .replace("<sup>", "^")
        .replace("&nbsp;", " ")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&quot;", "\"")
        .replace("&minus;", "-")
        .replace("&#39;", "'")
        .replace("\n\n", "\n")
        .replace("\n", "\n * ")
}
fn exists_problem_ids() -> HashSet<u32> {
    let solutions_mod_path = env::current_dir().unwrap()
        .join("src")
        .join("solutions")
        .join("mod.rs");
    let content = fs::read_to_string(solutions_mod_path).unwrap();
    let id_pattern = Regex::new(r"\w+_(\d{4})_").unwrap();
    id_pattern
        .captures_iter(&content)
        .map(|x| x.get(1).unwrap().as_str().parse().unwrap())
        .collect()
}
