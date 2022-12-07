use failure::{err_msg, Error};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::newline,
    combinator::{map, map_res},
    multi::many1,
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult,
};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum ListEntry {
    Directory(String),
    File(String, usize),
}

#[derive(Debug)]
pub enum DirectoryEntry {
    Directory(HashMap<String, DirectoryEntry>),
    File(usize),
}

#[derive(Debug)]
pub struct DirectorySizeEntry {
    size: usize,
    children: HashMap<String, DirectorySizeEntry>,
}

impl DirectoryEntry {
    fn dir_contents_mut(&mut self) -> Option<&mut HashMap<String, DirectoryEntry>> {
        match self {
            DirectoryEntry::Directory(contents) => Some(contents),
            DirectoryEntry::File(_) => None,
        }
    }

    fn dir_contents(&self) -> Option<&HashMap<String, DirectoryEntry>> {
        match self {
            DirectoryEntry::Directory(contents) => Some(contents),
            DirectoryEntry::File(_) => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    ChangeDirectory(String),
    ListDirectory(Box<[ListEntry]>),
}

fn basename(input: &str) -> IResult<&str, String> {
    map(take_while1(|c: char| !c.is_whitespace()), |name: &str| {
        name.to_string()
    })(input)
}

fn filesize(input: &str) -> IResult<&str, usize> {
    map_res(take_while1(|c: char| c.is_ascii_digit()), |size: &str| {
        size.parse()
    })(input)
}

fn directory_entry(input: &str) -> IResult<&str, ListEntry> {
    map(preceded(tag("dir "), basename), ListEntry::Directory)(input)
}

fn file_entry(input: &str) -> IResult<&str, ListEntry> {
    map(
        separated_pair(filesize, tag(" "), basename),
        |(size, name)| ListEntry::File(name, size),
    )(input)
}

fn entry(input: &str) -> IResult<&str, ListEntry> {
    terminated(alt((directory_entry, file_entry)), newline)(input)
}

fn entries(input: &str) -> IResult<&str, Box<[ListEntry]>> {
    map(many1(entry), Vec::into_boxed_slice)(input)
}

fn cd_command(input: &str) -> IResult<&str, Command> {
    map(
        delimited(tag("cd "), basename, newline),
        Command::ChangeDirectory,
    )(input)
}

fn ls_command(input: &str) -> IResult<&str, Command> {
    map(preceded(tag("ls\n"), entries), Command::ListDirectory)(input)
}

fn command(input: &str) -> IResult<&str, Command> {
    preceded(tag("$ "), alt((cd_command, ls_command)))(input)
}

fn commands(input: &str) -> IResult<&str, Box<[Command]>> {
    map(many1(command), Vec::into_boxed_slice)(input)
}

fn build_filesystem(commands: &[Command]) -> DirectoryEntry {
    let mut root = DirectoryEntry::Directory(HashMap::new());
    let mut cwd = vec![];

    for command in commands {
        match command {
            Command::ChangeDirectory(dirname) => {
                if dirname == ".." {
                    cwd.pop();
                } else if dirname == "/" {
                    cwd = vec![];
                } else if let Some(relative) = dirname.strip_prefix('/'){
                    cwd = relative.split('/').collect();
                } else {
                    cwd.extend(dirname.split('/'));
                }
            }
            Command::ListDirectory(list_output) => {
                let mut contents = root.dir_contents_mut().unwrap();
                for dirname in cwd.iter() {
                    contents = contents
                        .entry(dirname.to_string())
                        .or_insert_with(|| DirectoryEntry::Directory(HashMap::new()))
                        .dir_contents_mut()
                        .unwrap();
                }

                for entry in list_output.iter() {
                    let (name, data) = match entry {
                        ListEntry::File(name, size) => (name.clone(), DirectoryEntry::File(*size)),
                        ListEntry::Directory(name) => {
                            (name.clone(), DirectoryEntry::Directory(HashMap::new()))
                        }
                    };
                    contents.insert(name, data);
                }
            }
        }
    }

    root
}

fn get_directory_sizes(filesystem: &HashMap<String, DirectoryEntry>) -> DirectorySizeEntry {
    let children = filesystem
        .iter()
        .filter_map(|(name, entry)| {
            entry
                .dir_contents()
                .map(|contents| (name.clone(), get_directory_sizes(contents)))
        })
        .collect::<HashMap<_, _>>();

    let file_sizes: usize = filesystem
        .values()
        .filter_map(|entry| {
            if let DirectoryEntry::File(size) = entry {
                Some(size)
            } else {
                None
            }
        })
        .sum();
    let dir_sizes: usize = children.values().map(|entry| entry.size).sum();
    let size = file_sizes + dir_sizes;

    DirectorySizeEntry { size, children }
}

fn find_directory_sizes<F>(dir_sizes: &DirectorySizeEntry, filter: F) -> Vec<usize>
where
    F: Fn(&str, &DirectorySizeEntry) -> bool,
{
    let mut stack = vec![("/".to_string(), dir_sizes)];
    let mut sizes = vec![];

    while let Some((path, directory)) = stack.pop() {
        if filter(&path, directory) {
            sizes.push(directory.size);
        }

        for (name, child) in directory.children.iter() {
            stack.push((format!("{}{}/", path, name), child));
        }
    }

    sizes
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Command]>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        commands(&data)
            .map_err(|err| err_msg(format!("Failed to parse commands: {}", err)))
            .and_then(|(rest, commands)| {
                if rest.is_empty() {
                    Ok(commands)
                } else {
                    Err(err_msg(format!("Unparsed input: {:?}", rest)))
                }
            })
    }

    fn solve(commands: Self::Problem) -> (Option<String>, Option<String>) {
        let filesystem = build_filesystem(&commands);
        let dir_sizes = get_directory_sizes(filesystem.dir_contents().unwrap());
        let part_one = find_directory_sizes(&dir_sizes, |_, dir| dir.size <= 100_000)
            .iter()
            .sum::<usize>();

        let needed_size = 30_000_000 - (70_000_000 - dir_sizes.size);
        let part_two = find_directory_sizes(&dir_sizes, |_, dir| dir.size >= needed_size)
            .iter()
            .min()
            .unwrap()
            .to_string();
        (Some(part_one.to_string()), Some(part_two))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_ls() {
        let data = "$ ls\n268495 jgfbgjdb\ndir ltcqgnc\n272455 pct.bbd\n200036 phthcq\n174378 qld\ndir rbmstsf\n130541 trhbvp.fmm\ndir twjcmp\n";
        assert_eq!(
            command(data),
            Ok((
                "",
                Command::ListDirectory(Box::new([
                    ListEntry::File("jgfbgjdb".to_string(), 268495),
                    ListEntry::Directory("ltcqgnc".to_string()),
                    ListEntry::File("pct.bbd".to_string(), 272455),
                    ListEntry::File("phthcq".to_string(), 200036),
                    ListEntry::File("qld".to_string(), 174378),
                    ListEntry::Directory("rbmstsf".to_string()),
                    ListEntry::File("trhbvp.fmm".to_string(), 130541),
                    ListEntry::Directory("twjcmp".to_string()),
                ]))
            ))
        );
    }

    #[test]
    fn test_parse_ls_example() {
        let data = "$ ls\n4060174 j\n8033020 d.log\n5626152 d.ext\n7214296 k\n";
        assert_eq!(
            command(data),
            Ok((
                "",
                Command::ListDirectory(Box::new([
                    ListEntry::File("j".to_string(), 4060174),
                    ListEntry::File("d.log".to_string(), 8033020),
                    ListEntry::File("d.ext".to_string(), 5626152),
                    ListEntry::File("k".to_string(), 7214296),
                ]))
            ))
        );
    }
}
