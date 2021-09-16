
use std::borrow::Cow;

use regex::Regex;

use super::init::{
    co_initialize,
};
use super::bindings::{
    Windows::Win32::UI::Shell::PathCchCanonicalizeEx,
    Windows::Win32::Foundation::PWSTR,
};

lazy_static! {
    static ref ROOTED_MING_W64_COMPAT: Regex = Regex::new(r#"^/([a-zA-Z])/(.*)$"#).unwrap();
    static ref ROOTED_TILDE_COMPAT: Regex = Regex::new(r#"^(~)(.*)$"#).unwrap();
    static ref NORMALIZE_SLASH: Regex = Regex::new(r#"([\u{005C}\u{002F}]{1,})"#).unwrap();
}


/*
 * Boilerplate so I don't need to think about
 * types or borrowing
 *
 */
trait ToCow<'a> {
    fn to_cow(self) -> Cow<'a, str>;
}
impl<'a> ToCow<'a> for &'a str {
    fn to_cow(self) -> Cow<'a,str> {
        Cow::Borrowed(self)
    }
}
impl<'a> ToCow<'a> for String {
    fn to_cow(self) -> Cow<'a,str> {
        Cow::Owned(self)
    }
}
impl<'a> ToCow<'a> for &'a String {
    fn to_cow(self) -> Cow<'a,str> {
        Cow::Borrowed(self.as_str())
    }
}
impl<'a> ToCow<'a> for Cow<'a,str> {
    fn to_cow(self) -> Cow<'a,str> {
        self
    }
}
impl<'a> ToCow<'a> for &'a Cow<'_,str> {
    fn to_cow(self) -> Cow<'a,str> {
        Cow::Borrowed(self.as_ref())
    }
}




fn fix_root<'a,T>(arg: T) -> Result<Cow<'a,str>,Box<dyn std::error::Error>>
where
    T: ToCow<'a>
{
    let cow = <T as ToCow>::to_cow(arg);
    match ROOTED_MING_W64_COMPAT.captures(&cow) {
        Option::None => Ok(cow),
        Option::Some(caps) => {
            let drive_letter = caps.get(1).unwrap().as_str().to_uppercase();
            let rest = caps.get(2).unwrap().as_str();
            Ok(Cow::Owned(format!(r#"{}:\{}"#,drive_letter, rest)))
        }
    }
}

#[test]
fn test_fix_root() {

    // sanity check
    assert_eq!(fix_root(r#"F:\Users\Valarauca"#).unwrap(), r#"F:\Users\Valarauca"#);
    assert_eq!(fix_root(r#"/f/Users/Valarauca"#).unwrap(), r#"F:\Users/Valarauca"#);

    // terminating slash
    assert_eq!(fix_root(r#"F:\Users\Valarauca\"#).unwrap(), r#"F:\Users\Valarauca\"#);
    assert_eq!(fix_root(r#"/f/Users/Valarauca\"#).unwrap(), r#"F:\Users/Valarauca\"#);

    // opposite slash
    assert_eq!(fix_root(r#"F:\Users\Valarauca/"#).unwrap(), r#"F:\Users\Valarauca/"#);
    assert_eq!(fix_root(r#"/f/Users/Valarauca/"#).unwrap(), r#"F:\Users/Valarauca/"#);

    // double terminating slash
    assert_eq!(fix_root(r#"F:\Users\Valarauca\\"#).unwrap(), r#"F:\Users\Valarauca\\"#);
    assert_eq!(fix_root(r#"/f/Users/Valarauca\\"#).unwrap(), r#"F:\Users/Valarauca\\"#);

    // double opposite slash
    assert_eq!(fix_root(r#"F:\Users\Valarauca//"#).unwrap(), r#"F:\Users\Valarauca//"#);
    assert_eq!(fix_root(r#"/f/Users/Valarauca//"#).unwrap(), r#"F:\Users/Valarauca//"#);
}

fn fix_tilde<'a,T>(arg: T) -> Result<Cow<'a,str>,Box<dyn std::error::Error>>
where
    T: ToCow<'a>
{
    let cow = <T as ToCow>::to_cow(arg);
    if ROOTED_TILDE_COMPAT.is_match(cow.as_ref()) {
        let home = std::env::var("HOME")?;
        Ok(ROOTED_TILDE_COMPAT.replace_all(&cow, format!("{}$2", home))
           .to_string().to_cow())
    } else {
        Ok(cow)
    }
}

#[test]
fn test_fix_tilde() {

    // test cases which should be uneffected
    assert_eq!(fix_tilde(r#"C:\Users\Valarauca\Documents\"#).unwrap(),r#"C:\Users\Valarauca\Documents\"#);
    assert_eq!(fix_tilde(r#"C:\Users\\\Valarauca\Documents\"#).unwrap(),r#"C:\Users\\\Valarauca\Documents\"#);
    assert_eq!(fix_tilde(r#"~/Documents/"#).unwrap(),r#"C:\Users\valarauca/Documents/"#);

    // trivial cases
    assert_eq!(fix_tilde(r#"~/"#).unwrap(), r#"C:\Users\valarauca/"#);
    assert_eq!(fix_tilde(r#"~\"#).unwrap(), r#"C:\Users\valarauca\"#);
    assert_eq!(fix_tilde(r#"~///"#).unwrap(), r#"C:\Users\valarauca///"#);
    assert_eq!(fix_tilde(r#"~\\\lol\"#).unwrap(), r#"C:\Users\valarauca\\\lol\"#);
}

fn normalize_slash<'a,T>(arg: T) -> Result<Cow<'a,str>,Box<dyn std::error::Error>>
where
    T: ToCow<'a>
{
    let cow = <T as ToCow>::to_cow(arg);
    if NORMALIZE_SLASH.is_match(cow.as_ref()) {
        Ok(NORMALIZE_SLASH.replace_all(&cow, r#"\"#)
            .to_string()
            .to_cow())
    } else {
        Ok(cow)
    }
}

#[test]
fn test_normalize_slash() {
    // test cases which should be uneffected
    assert_eq!(normalize_slash(r#"C:\Users\Valarauca\Documents\"#).unwrap(),r#"C:\Users\Valarauca\Documents\"#);

    // simple test cases
    assert_eq!(normalize_slash(r#"C:/Users/Valarauca/Documents/"#).unwrap(),r#"C:\Users\Valarauca\Documents\"#);
    assert_eq!(normalize_slash(r#"C:\Users\\\\Valarauca\\Documents\\/\"#).unwrap(),r#"C:\Users\Valarauca\Documents\"#);
    assert_eq!(normalize_slash(r#"C:\Users/Valarauca\/\Documents\\/\"#).unwrap(),r#"C:\Users\Valarauca\Documents\"#);
}

fn path_cch_canonicalize_ex<'a,T>(arg: T) -> Result<Cow<'a,str>,Box<dyn std::error::Error>>
where
    T: ToCow<'a>
{
    co_initialize()?;

    let cow = <T as ToCow>::to_cow(arg);

    // 32KiB is totally unreasonable for a path length
    #[allow(non_snake_case)]
    let KiB32 = 32768usize;
    let mut v = Vec::<u16>::with_capacity(KiB32);
    for _ in 0..KiB32 {
        v.push(0u16);
    }

    unsafe {
        PathCchCanonicalizeEx(PWSTR(v.as_mut_ptr()), KiB32, cow.as_ref(), 1)?
    };

    let mut length = 0usize;
    for index in 0..KiB32 {
        if v[index] == 0 {
            break;
        }
        length+=1;
    }
    Ok(String::from_utf16(&v.as_slice()[0..length])?.to_cow())
}

#[test]
fn test_path_cch_canonicalize_ex() {
    assert_eq!(path_cch_canonicalize_ex(r#"C:\Users\Valarauca\Documents\"#).unwrap(), r#"C:\Users\Valarauca\Documents\"#);
    assert_eq!(path_cch_canonicalize_ex(r#"C:\Users\Valarauca\Documents\..\..\"#).unwrap(), r#"C:\Users\"#);
}


pub fn canonicalize(path: &str) -> Result<String,Box<dyn std::error::Error>> {

    let a = fix_root(path)?;
    let b = fix_tilde(a)?;
    let c = normalize_slash(b)?;
    let d = path_cch_canonicalize_ex(c)?;
    Ok(d.to_string())
}

#[test]
fn assert_matches() {

    assert_eq!(canonicalize("~/Documents/").unwrap(), r#"C:\Users\valarauca\Documents\"#);
    assert_eq!(canonicalize("/f/Downloads/").unwrap(), r#"F:\Downloads\"#);
    assert_eq!(canonicalize("/f/Downloads/../").unwrap(), r#"F:\"#);

}
