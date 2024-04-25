use pyo3::prelude::*;
use pyo3::types::{PyNone, PyTuple};
use regex;

#[pyclass(module = "regexrs")]
struct Match {
    #[pyo3(get)]
    string: String,
    #[pyo3(get)]
    re: Pattern,
    #[pyo3(get)]
    pos: usize,
    #[pyo3(get)]
    endpos: usize,
}

#[pymethods]
impl Match {
    #[pyo3(signature = (*args))]
    fn group(&self, py: Python<'_>, args: &PyTuple) -> PyResult<PyObject> {
        let caps = self
            .re
            .regex
            .captures(&self.string)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("No match found"))?;

        if args.is_empty() {
            // No arguments, return the whole match (group 0)
            return Ok(caps
                .get(0)
                .map_or_else(|| py.None(), |m| m.as_str().into_py(py)));
        }

        let groups: Vec<PyObject> = args
            .iter()
            .map(|g| match g.extract::<usize>() {
                Ok(index) => caps
                    .get(index)
                    .map_or_else(|| py.None(), |m| m.as_str().into_py(py)),
                Err(_) => py.None(),
            })
            .collect();

        if groups.len() == 1 {
            Ok(groups[0].clone_ref(py))
        } else {
            let tuple = PyTuple::new(py, &groups);
            Ok(tuple.to_object(py))
        }
    }

    fn groups(&self, py: Python<'_>) -> PyResult<PyObject> {
        let caps = self
            .re
            .regex
            .captures(&self.string)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("No match found"))?;

        let groups: Vec<PyObject> = caps
            .iter()
            .skip(1) // Skip the entire match which is at index 0
            .map(|m| match m {
                Some(matched) => matched.as_str().into_py(py),
                None => py.None(),
            })
            .collect();

        Ok(PyTuple::new(py, &groups).to_object(py))
    }
}

#[pyclass(module = "regexrs")]
#[derive(Clone)]
struct Pattern {
    // #[pyo3(get)]
    // flags: i32,
    regex: regex::Regex,
}

impl Pattern {
    fn new(pattern: String) -> Self {
        Pattern {
            regex: regex::Regex::new(pattern.as_str()).unwrap(),
        }
    }
}

#[pymethods]
impl Pattern {
    pub fn r#match(&self, string: String, pos: Option<usize>) -> PyResult<Option<Match>> {
        let p = pos.unwrap_or(0);
        let m = self.regex.find_at(string.as_str(), p);
        match m {
            Some(matched) if matched.start() == p => {
                let r = Match {
                    string: String::from(matched.as_str()),
                    re: self.clone(),
                    pos: matched.start(),
                    endpos: matched.end(),
                };
                Ok(Some(r))
            }
            _ => Ok(None),
        }
    }
}

fn python_regex_flags_to_inline(pattern: String, flags: i32) -> String {
    // Define the flags based on the Python re module flag values
    const IGNORECASE: i32 = 2; // re.I or re.IGNORECASE
    const MULTILINE: i32 = 8; // re.M or re.MULTILINE
    const DOTALL: i32 = 16; // re.S or re.DOTALL
    const VERBOSE: i32 = 64; // re.X or re.VERBOSE

    let mut result = String::new();
    let mut flags_applied: i32 = 0;
    // Start the inline flag string
    result.push_str("(?");

    if flags & IGNORECASE != 0 {
        result.push('i');
        flags_applied = flags_applied + 1;
    }
    if flags & MULTILINE != 0 {
        result.push('m');
        flags_applied = flags_applied + 1;
    }
    if flags & DOTALL != 0 {
        result.push('s');
        flags_applied = flags_applied + 1;
    }
    if flags & VERBOSE != 0 {
        result.push('x');
        flags_applied = flags_applied + 1;
    }

    // Close the inline flag string
    result.push(')');

    // Return the resulting inline flags or an empty string if no flags are set
    if flags_applied >= 1 {
        return format!("{}{}", result, pattern);
    } else {
        return pattern;
    }
}

#[pyfunction]
fn compile(pattern: String, flags: Option<i32>) -> Pattern {
    match flags {
        Some(given_flags) => Pattern::new(python_regex_flags_to_inline(pattern, given_flags)),
        None => Pattern::new(pattern),
    }
}

#[pyfunction]
fn findall(
    py: Python,
    pattern: PyObject,
    string: String,
    flags: Option<i32>,
) -> PyResult<Vec<String>> {
    let re: regex::Regex = if let Ok(s) = pattern.extract::<&str>(py) {
        match flags {
            Some(given_flags) => {
                regex::Regex::new(python_regex_flags_to_inline(s.to_string(), given_flags).as_str())
                    .map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                            "Invalid regex pattern: {}",
                            e
                        ))
                    })?
            }
            None => regex::Regex::new(s).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Invalid regex pattern: {}",
                    e
                ))
            })?,
        }
    } else if let Ok(pat) = pattern.extract::<Pattern>(py) {
        match flags {
            Some(_) => {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Cannot use flags with compiled pattern",
                ));
            }
            None => pat.regex,
        }
    } else {
        // Neither a string nor a Pattern object
        return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Pattern must be a string or a Pattern object",
        ));
    };

    // Using the Regex object to find all matches
    Ok(re
        .find_iter(&string)
        .map(|mat| mat.as_str().to_string())
        .collect())
}

#[pyfunction]
fn r#match(
    py: Python,
    pattern: PyObject,
    string: String,
    flags: Option<i32>,
) -> PyResult<Option<Match>> {
    let re: regex::Regex = if let Ok(s) = pattern.extract::<&str>(py) {
        match flags {
            Some(given_flags) => {
                regex::Regex::new(python_regex_flags_to_inline(s.to_string(), given_flags).as_str())
                    .map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                            "Invalid regex pattern: {}",
                            e
                        ))
                    })?
            }
            None => regex::Regex::new(s).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Invalid regex pattern: {}",
                    e
                ))
            })?,
        }
    } else if let Ok(pat) = pattern.extract::<Pattern>(py) {
        match flags {
            Some(_) => {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Cannot use flags with compiled pattern",
                ));
            }
            None => pat.regex,
        }
    } else {
        // Neither a string nor a Pattern object
        return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Pattern must be a string or a Pattern object",
        ));
    };

    // Using the Regex object to find all matches
    let m = re.find(string.as_str());
    match m {
        Some(matched) if matched.start() == 0 => {
            let r = Match {
                string: String::from(matched.as_str()),
                re: Pattern {regex: re}, // XXX: if we are passed a Pattern object, we should use that instead of creating a new one
                pos: matched.start(),
                endpos: matched.end(),
            };
            Ok(Some(r))
        }
        _ => Ok(None),
    }
}


#[pyfunction]
fn fullmatch(
    py: Python,
    pattern: PyObject,
    string: String,
    flags: Option<i32>,
) -> PyResult<Option<Match>> {
    let re: regex::Regex = if let Ok(s) = pattern.extract::<&str>(py) {
        match flags {
            Some(given_flags) => {
                regex::Regex::new(python_regex_flags_to_inline(s.to_string(), given_flags).as_str())
                    .map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                            "Invalid regex pattern: {}",
                            e
                        ))
                    })?
            }
            None => regex::Regex::new(s).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Invalid regex pattern: {}",
                    e
                ))
            })?,
        }
    } else if let Ok(pat) = pattern.extract::<Pattern>(py) {
        match flags {
            Some(_) => {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Cannot use flags with compiled pattern",
                ));
            }
            None => pat.regex,
        }
    } else {
        // Neither a string nor a Pattern object
        return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Pattern must be a string or a Pattern object",
        ));
    };

    // Using the Regex object to find all matches
    let m = re.find(string.as_str());
    match m {
        Some(matched) if (matched.start() == 0 && matched.end() == string.len()) => {
            let r = Match {
                string: String::from(matched.as_str()),
                re: Pattern {regex: re}, // XXX: if we are passed a Pattern object, we should use that instead of creating a new one
                pos: matched.start(),
                endpos: matched.end(),
            };
            Ok(Some(r))
        }
        _ => Ok(None),
    }
}

#[pymodule]
#[pyo3(name = "regexrs")]
fn regexrs(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Pattern>()?;
    m.add_class::<Match>()?;
    m.add("NOFLAG", 0);
    m.add("IGNORECASE", 2);
    m.add("I", 2);
    m.add("MULTILINE", 8);
    m.add("M", 8);
    m.add("DOTALL", 16);
    m.add("S", 16);
    m.add("VERBOSE", 64);
    m.add("X", 64);
    let compile_func = wrap_pyfunction!(compile, m)?;
    let findall_func = wrap_pyfunction!(findall, m)?;
    let match_func = wrap_pyfunction!(r#match, m)?;
    let fullmatch_func = wrap_pyfunction!(fullmatch, m)?;
    compile_func.setattr("__module__", "regexrs");
    findall_func.setattr("__module__", "regexrs");
    match_func.setattr("__module__", "regexrs");
    fullmatch_func.setattr("__module__", "regexrs");
    m.add_function(compile_func)?;
    m.add_function(findall_func)?;
    m.add_function(match_func)?;
    m.add_function(fullmatch_func)?;
    Ok(())
}
