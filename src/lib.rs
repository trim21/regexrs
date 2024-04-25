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
    #[pyo3(get)]
    lastgroup: Option<String>,
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

    fn end(&self) -> PyResult<usize> {
        return Ok(self.endpos);
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<regexrs.Match object; span=({}, {}), match={:?}>", self.pos, self.endpos, self.string))
    }
}

#[pyclass(module = "regexrs")]
#[derive(Clone)]
struct Pattern {
    regex: regex::Regex,
}

impl Pattern {
    fn new(pattern: &str) -> Self {
        Pattern {
            regex: regex::Regex::new(pattern).unwrap(),
        }
    }
}

fn get_byte_to_code_point_and_reverse(haystack: &str) -> (Vec<usize>, Vec<usize>) {
    // Map UTF-8 byte index to Unicode code point index; the latter is what
    // Python users expect.
    let mut byte_to_code_point = vec![usize::MAX; haystack.len() + 1];
    let mut code_point_to_byte = vec![usize::MAX; haystack.len() + 1];
    let mut max_codepoint = 0;
    for (codepoint_off, (byte_off, _)) in haystack.char_indices().enumerate() {
        byte_to_code_point[byte_off] = codepoint_off;
        code_point_to_byte[codepoint_off] = byte_off;
        max_codepoint = codepoint_off;
    }
    // End index is exclusive (e.g. 0:3 is first 3 characters), so handle
    // the case where pattern is at end of string.
    if !haystack.is_empty() {
        byte_to_code_point[haystack.len()] = max_codepoint + 1;
    }
    (byte_to_code_point, code_point_to_byte)
}

fn get_byte_to_code_point(haystack: &str) -> Vec<usize> {
    // Map UTF-8 byte index to Unicode code point index; the latter is what
    // Python users expect.
    let mut byte_to_code_point = vec![usize::MAX; haystack.len() + 1];
    let mut max_codepoint = 0;
    for (codepoint_off, (byte_off, _)) in haystack.char_indices().enumerate() {
        byte_to_code_point[byte_off] = codepoint_off;
        max_codepoint = codepoint_off;
    }
    // End index is exclusive (e.g. 0:3 is first 3 characters), so handle
    // the case where pattern is at end of string.
    if !haystack.is_empty() {
        byte_to_code_point[haystack.len()] = max_codepoint + 1;
    }
    byte_to_code_point
}

#[pymethods]
impl Pattern {

    pub fn r#match(&self, string: String, pos: Option<usize>) -> PyResult<Option<Match>> {
        let (byte_to_code_point, code_point_to_byte) = get_byte_to_code_point_and_reverse(string.as_str());
        let p = code_point_to_byte[pos.unwrap_or(0)];
        if let Some(caps) = self.regex.captures_at(&string, p) {
            if let Some(matched) = caps.get(0) {
                if matched.start() == p {
                    // Extract the name of the last matched group
                    let last_group_name = self.regex.capture_names()
                        .filter_map(|name| name) // Skip None values for unnamed groups
                        .filter_map(|name| {
                            // Only consider the group if it has a match
                            caps.name(name).map(|_| name.to_string())
                        })
                        .last(); // Get the last group that had a match


                    return Ok(Some(Match {
                        string: String::from(matched.as_str()),
                        re: self.clone(),
                        pos: byte_to_code_point[matched.start()],
                        endpos: byte_to_code_point[matched.end()],
                        lastgroup: last_group_name,
                    }));
                }
            }
        }
        Ok(None) // No match found or the match does not start at 'p'
    }


    fn __repr__(&self) -> PyResult<String> {
        // TODO: form a raw string repr
        Ok(format!("regexrs.compile({:?})", self.regex.as_str()))
    }
}

fn python_regex_flags_to_inline(pattern: &str, flags: i32) -> String {
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
    if flags_applied > 0 {
        return format!("{}{}", result, pattern);
    } else {
        return pattern.to_owned();
    }
}

#[pyfunction]
fn compile(pattern: &str, flags: Option<i32>) -> Pattern {
    match flags {
        Some(given_flags) => Pattern::new(python_regex_flags_to_inline(pattern, given_flags).as_str()),
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
                regex::Regex::new(python_regex_flags_to_inline(s, given_flags).as_str())
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
                regex::Regex::new(python_regex_flags_to_inline(s, given_flags).as_str())
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
        return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Pattern must be a string or a Pattern object"));
    };


    if let Some(caps) = re.captures(&string) {
        if let Some(matched) = caps.get(0) {
            // Check that the match starts exactly at the position `p`
            if matched.start() == 0 {
                let last_group_name = re.capture_names()
                    .filter_map(|name| name)
                    .filter_map(|name| caps.name(name).map(|_| name.to_string()))
                    .last();
                let byte_to_code_point = get_byte_to_code_point(&string);

                return Ok(Some(Match {
                    string: String::from(matched.as_str()),
                    re: Pattern { regex: re },
                    pos: byte_to_code_point[matched.start()],
                    endpos: byte_to_code_point[matched.end()],
                    lastgroup: last_group_name,
                }));
            }
        }
    }
    Ok(None) // No valid match found or the match does not start at 'p'
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
                regex::Regex::new(python_regex_flags_to_inline(s, given_flags).as_str())
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
        return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Pattern must be a string or a Pattern object"));
    };

    if let Some(caps) = re.captures(&string) {
        if let Some(matched) = caps.get(0) {
            if matched.start() == 0 && matched.end() == string.len() {
                let last_group_name = re.capture_names()
                    .filter_map(|name| name)
                    .filter_map(|name| caps.name(name).map(|_| name.to_string()))
                    .last();
                let byte_to_code_point = get_byte_to_code_point(&string);

                return Ok(Some(Match {
                    string: String::from(matched.as_str()),
                    re: Pattern { regex: re },
                    pos: byte_to_code_point[matched.start()],
                    endpos: byte_to_code_point[matched.end()],
                    lastgroup: last_group_name,
                }));
            }
        }
    }
    Ok(None)
}


#[pyfunction]
fn escape(pattern: &str) -> PyResult<String> {
    Ok(regex::escape(pattern))
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
    let escape_func = wrap_pyfunction!(escape, m)?;
    compile_func.setattr("__module__", "regexrs");
    findall_func.setattr("__module__", "regexrs");
    match_func.setattr("__module__", "regexrs");
    fullmatch_func.setattr("__module__", "regexrs");
    escape_func.setattr("__module__", "regexrs");
    m.add_function(compile_func)?;
    m.add_function(findall_func)?;
    m.add_function(match_func)?;
    m.add_function(fullmatch_func)?;
    m.add_function(escape_func)?;
    Ok(())
}
