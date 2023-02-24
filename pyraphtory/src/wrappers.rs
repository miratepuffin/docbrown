use pyo3::prelude::*;

use dbc::tgraph_shard;
use docbrown_core as dbc;

#[pyclass]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Direction {
    OUT,
    IN,
    BOTH,
}

impl From<Direction> for dbc::Direction {
    fn from(d: Direction) -> dbc::Direction {
        match d {
            Direction::OUT => dbc::Direction::OUT,
            Direction::IN => dbc::Direction::IN,
            Direction::BOTH => dbc::Direction::BOTH,
        }
    }
}

#[derive(FromPyObject, Debug)]
pub enum Prop {
    Str(String),
    I64(i64),
    U64(u64),
    F64(f64),
    Bool(bool),
}

impl From<Prop> for dbc::Prop {
    fn from(prop: Prop) -> dbc::Prop {
        match prop {
            Prop::Str(string) => dbc::Prop::Str(string.clone()),
            Prop::I64(i64) => dbc::Prop::I64(i64),
            Prop::U64(u64) => dbc::Prop::U64(u64),
            Prop::F64(f64) => dbc::Prop::F64(f64),
            Prop::Bool(bool) => dbc::Prop::Bool(bool),
        }
    }
}

#[pyclass]
pub struct TEdge {
    #[pyo3(get)]
    pub src: u64,
    #[pyo3(get)]
    pub dst: u64,
    #[pyo3(get)]
    pub t: Option<i64>,
    #[pyo3(get)]
    pub is_remote: bool,
}

impl From<tgraph_shard::TEdge> for TEdge {
    fn from(value: tgraph_shard::TEdge) -> Self {
        let tgraph_shard::TEdge {
            src,
            dst,
            t,
            is_remote,
        } = value;
        TEdge {
            src,
            dst,
            t,
            is_remote,
        }
    }
}

#[pyclass]
pub struct TVertex {
    #[pyo3(get)]
    pub g_id: u64,
}

impl From<tgraph_shard::TVertex> for TVertex {
    fn from(value: tgraph_shard::TVertex) -> TVertex {
        let tgraph_shard::TVertex { g_id, .. } = value;
        TVertex { g_id }
    }
}

#[pyclass]
pub struct VertexIdsIterator {
    pub(crate) iter: Box<dyn Iterator<Item = u64> + Send>,
}

#[pymethods]
impl VertexIdsIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<u64> {
        slf.iter.next()
    }
}

#[pyclass]
pub struct EdgeIterator {
    pub(crate) iter: Box<dyn Iterator<Item = TEdge> + Send>,
}

#[pymethods]
impl EdgeIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<TEdge> {
        slf.iter.next()
    }
}
