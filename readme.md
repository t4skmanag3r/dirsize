# dirsize
dirsize is a disk usage scanner with a navigable TUI (terminal-user-interface) for dispalying directory/file sizes written in rust

## Instalation and usage
### using cargo :
`cargo install dirsize`

run - `dirsize [options] <PATH>`

arguments: 
- PATH : path to dirrectory

options :
- -s --size \<SIZE> : size format, possible values : [gb, mb, kb, b] [default: mb]
- -h --help : shows about, usage information
- -V --version : show version

note:
if you want to use cargo use [rustup](https://www.rust-lang.org/learn/get-started) to install it

### run via executable (windows) :
download the executable from [realeases](https://github.com/t4skmanag3r/dirsize/releases)

run - `dirsize [path to directory to scan]` from the download location, within a command-line tool (cmd, bash, etc...)
or [add the executable to PATH environment variables](https://learn.microsoft.com/en-us/previous-versions/office/developer/sharepoint-2010/ee537574(v=office.14)) to run from anywhere


## links
- [crates.io](https://crates.io/crates/dirsize)
- [github](https://github.com/t4skmanag3r/dirsize)



