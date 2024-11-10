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
    fn group(&self, py: Python<'_>, args: &Bound<'_, PyTuple>) -> PyResult<PyObject> {
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
            let tuple = PyTuple::new_bound(py, &groups);
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

        Ok(PyTuple::new_bound(py, &groups).to_object(py))
    }

    fn end(&self) -> PyResult<usize> {
        return Ok(self.endpos);
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "<regexrs.Match object; span=({}, {}), match={:?}>",
            self.pos, self.endpos, self.string
        ))
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
    // based on https://github.com/G-Research/ahocorasick_rs/blob/034e3f67e12198c08137bb9fb3153cb01cf5da31/src/lib.rs#L72-L88
    // modified to provide additional reverse mapping

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
    // copied from https://github.com/G-Research/ahocorasick_rs/blob/034e3f67e12198c08137bb9fb3153cb01cf5da31/src/lib.rs#L72-L88

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
    #[pyo3(signature=(string,pos=None))]
    pub fn r#match(&self, string: String, pos: Option<usize>) -> PyResult<Option<Match>> {
        if string.is_empty() {
            return Ok(None);
        }

        let (byte_to_code_point, code_point_to_byte) =
            get_byte_to_code_point_and_reverse(string.as_str());
        let p = code_point_to_byte[pos.unwrap_or(0)];
        if let Some(caps) = self.regex.captures_at(&string, p) {
            if let Some(matched) = caps.get(0) {
                if matched.start() == p {
                    // Extract the name of the last matched group
                    let last_group_name = self
                        .regex
                        .capture_names()
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

    fn findall(&self, string: String, flags: Option<i32>) -> PyResult<Vec<String>> {
        Ok(self
            .regex
            .find_iter(&string)
            .map(|mat| mat.as_str().to_string())
            .collect())
    }

    fn __repr__(&self) -> PyResult<String> {
        // TODO: form a raw string repr
        Ok(format!("regexrs.compile({:?})", self.regex.as_str()))
    }
}

#[pyfunction]
#[pyo3(signature = (pattern, flags=None))]
fn compile(pattern: &str, flags: Option<i32>) -> Pattern {
    Pattern::new(pattern)
}

#[pyfunction]
fn escape(pattern: &str) -> PyResult<String> {
    Ok(regex::escape(pattern))
}

#[pymodule]
#[pyo3(name = "re_rs")]
fn re_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pattern>()?;
    m.add_class::<Match>()?;
    m.add("NOFLAG", 0)?;
    m.add("IGNORECASE", 2)?;
    m.add("I", 2)?;
    m.add("MULTILINE", 8)?;
    m.add("M", 8)?;
    m.add("DOTALL", 16)?;
    m.add("S", 16)?;
    m.add("VERBOSE", 64)?;
    m.add("X", 64)?;
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(escape, m)?)?;
    Ok(())
}
