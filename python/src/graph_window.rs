use std::collections::HashMap;

use crate::wrappers;
use crate::{graph::Graph, wrappers::*};
use docbrown_db::graph_window;
use docbrown_db::view_api::*;
use itertools::Itertools;
use pyo3::prelude::*;

#[pyclass]
pub struct GraphWindowSet {
    window_set: graph_window::GraphWindowSet,
}

impl From<graph_window::GraphWindowSet> for GraphWindowSet {
    fn from(value: graph_window::GraphWindowSet) -> Self {
        GraphWindowSet::new(value)
    }
}

impl GraphWindowSet {
    pub fn new(window_set: graph_window::GraphWindowSet) -> GraphWindowSet {
        GraphWindowSet { window_set }
    }
}

#[pymethods]
impl GraphWindowSet {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>, py: Python) -> Option<WindowedGraph> {
        let windowed_graph = slf.window_set.next()?;
        Some(windowed_graph.into())
    }
}

#[pyclass]
pub struct WindowedGraph {
    pub(crate) graph_w: graph_window::WindowedGraph,
}

impl From<graph_window::WindowedGraph> for WindowedGraph {
    fn from(value: graph_window::WindowedGraph) -> Self {
        WindowedGraph { graph_w: value }
    }
}

impl WindowedGraph {
    pub fn new(graph: &Graph, t_start: i64, t_end: i64) -> Self {
        Self {
            graph_w: graph_window::WindowedGraph::new(graph.graph.clone(), t_start, t_end),
        }
    }
}

#[pymethods]
impl WindowedGraph {
    //******  Metrics APIs ******//

    pub fn earliest_time(&self) -> Option<i64> {
        self.graph_w.earliest_time()
    }

    pub fn latest_time(&self) -> Option<i64> {
        self.graph_w.latest_time()
    }

    pub fn num_edges(&self) -> usize {
        self.graph_w.num_edges()
    }

    pub fn num_vertices(&self) -> usize {
        self.graph_w.num_vertices()
    }

    pub fn has_vertex(&self, id: &PyAny) -> PyResult<bool> {
        let v = Graph::extract_id(id)?;
        Ok(self.graph_w.has_vertex(v))
    }

    pub fn has_edge(&self, src: &PyAny, dst: &PyAny) -> PyResult<bool> {
        let src = Graph::extract_id(src)?;
        let dst = Graph::extract_id(dst)?;
        Ok(self.graph_w.has_edge(src, dst))
    }

    //******  Getter APIs ******//

    pub fn vertex(slf: PyRef<'_, Self>, id: &PyAny) -> PyResult<Option<WindowedVertex>> {
        let v = Graph::extract_id(id)?;
        match slf.graph_w.vertex(v) {
            None => Ok(None),
            Some(v) => {
                let g: Py<Self> = slf.into();
                Ok(Some(WindowedVertex::new(g, v)))
            }
        }
    }

    pub fn __getitem__(slf: PyRef<'_, Self>, id: &PyAny) -> PyResult<Option<WindowedVertex>> {
        let v = Graph::extract_id(id)?;
        match slf.graph_w.vertex(v) {
            None => Ok(None),
            Some(v) => {
                let g: Py<Self> = slf.into();
                Ok(Some(WindowedVertex::new(g, v)))
            }
        }
    }

    pub fn vertex_ids(&self) -> VertexIdsIterator {
        VertexIdsIterator {
            iter: self.graph_w.vertices().id(),
        }
    }

    pub fn vertices(slf: PyRef<'_, Self>) -> WindowedVertices {
        let g: Py<Self> = slf.into();
        WindowedVertices { graph: g }
    }

    pub fn edge(&self, src: &PyAny, dst: &PyAny) -> PyResult<Option<WindowedEdge>> {
        let src = Graph::extract_id(src)?;
        let dst = Graph::extract_id(dst)?;
        Ok(self.graph_w.edge(src, dst).map(|we| we.into()))
    }

    pub fn edges(&self) -> WindowedEdgeIterator {
        WindowedEdgeIterator {
            iter: Box::new(self.graph_w.edges().map(|te| te.into())),
        }
    }
}

#[pyclass]
pub struct WindowedVertex {
    #[pyo3(get)]
    pub id: u64,
    pub(crate) graph: Py<WindowedGraph>,
    pub(crate) vertex_w: graph_window::WindowedVertex,
}

//TODO need to implement but would need to change a lot of things
//Have to rely on internal from for the moment
// impl From<graph_window::WindowedVertex> for WindowedVertex {
//     fn from(value: graph_window::WindowedVertex) ->WindowedVertex {
//
//     }
// }

impl WindowedVertex {
    fn from(&self, value: graph_window::WindowedVertex) -> WindowedVertex {
        WindowedVertex {
            id: value.id(),
            graph: self.graph.clone(),
            vertex_w: value,
        }
    }

    pub(crate) fn new(
        graph: Py<WindowedGraph>,
        vertex: graph_window::WindowedVertex,
    ) -> WindowedVertex {
        WindowedVertex {
            graph,
            id: vertex.id(),
            vertex_w: vertex,
        }
    }
}

#[pymethods]
impl WindowedVertex {

    pub fn __getitem__(&self, name: String) -> Option<Prop> {
        self.property(name,Some(true))
    }

    pub fn has_property(&self,name:String,include_static:Option<bool>) -> bool {
        match include_static {
            None => {self.vertex_w.has_property(name,true)}
            Some(b) => {self.vertex_w.has_property(name,b)}
        }
    }

    pub fn property(&self,name:String,include_static:Option<bool>) -> Option<Prop> {
        let res = match include_static {
            None => {self.vertex_w.property(name,true)}
            Some(b) => {self.vertex_w.property(name,b)}
        };

        match res{
            None => {None}
            Some(prop) => {Some(prop.into())}
        }
    }

    pub fn properties(&self,include_static:Option<bool>) -> HashMap<String,Prop> {
        match include_static {
            None => {self.vertex_w.properties(true)}
            Some(b) => {self.vertex_w.properties(b)}
        }.into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect()
    }

    pub fn property_names(&self,include_static:Option<bool>) -> Vec<String> {
        match include_static {
            None => {self.vertex_w.property_names(true)}
            Some(b) => {self.vertex_w.property_names(b)}
        }

    }

    pub fn property_history(&self,name:String) -> Vec<(i64, Prop)> {
        self.vertex_w.property_history(name).into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect()
    }

    pub fn property_histories(&self) -> HashMap<String, Vec<(i64, Prop)>> {
        self.vertex_w.property_histories().into_iter()
            .map(|(k, v)| (k, v.into_iter().map(|(t,p)| (t,p.into())).collect()))
            .collect()
    }

    pub fn has_static_property(&self,name:String)->bool {
        self.vertex_w.has_static_property(name)
    }
    pub fn static_property(&self,name:String)-> Option<Prop>{
        match self.vertex_w.static_property(name) {
            None => {None}
            Some(prop) => {Some(prop.into())}
        }
    }

    pub fn degree(&self, t_start: Option<i64>, t_end: Option<i64>) -> usize {
        match (t_start, t_end) {
            (None, None) => self.vertex_w.degree(),
            _ => self
                .vertex_w
                .degree_window(t_start.unwrap_or(i64::MIN), t_end.unwrap_or(i64::MAX)),
        }
    }
    pub fn in_degree(&self, t_start: Option<i64>, t_end: Option<i64>) -> usize {
        match (t_start, t_end) {
            (None, None) => self.vertex_w.in_degree(),
            _ => self
                .vertex_w
                .in_degree_window(t_start.unwrap_or(i64::MIN), t_end.unwrap_or(i64::MAX)),
        }
    }
    pub fn out_degree(&self, t_start: Option<i64>, t_end: Option<i64>) -> usize {
        match (t_start, t_end) {
            (None, None) => self.vertex_w.out_degree(),
            _ => self
                .vertex_w
                .out_degree_window(t_start.unwrap_or(i64::MIN), t_end.unwrap_or(i64::MAX)),
        }
    }
    pub fn edges(&self, t_start: Option<i64>, t_end: Option<i64>) -> WindowedEdgeIterator {
        match (t_start, t_end) {
            (None, None) => WindowedEdgeIterator {
                iter: Box::new(self.vertex_w.edges().map(|te| te.into())),
            },
            _ => WindowedEdgeIterator {
                iter: Box::new(
                    self.vertex_w
                        .edges_window(t_start.unwrap_or(i64::MIN), t_end.unwrap_or(i64::MAX))
                        .map(|te| te.into()),
                ),
            },
        }
    }

    pub fn in_edges(&self, t_start: Option<i64>, t_end: Option<i64>) -> WindowedEdgeIterator {
        match (t_start, t_end) {
            (None, None) => WindowedEdgeIterator {
                iter: Box::new(self.vertex_w.in_edges().map(|te| te.into())),
            },
            _ => WindowedEdgeIterator {
                iter: Box::new(
                    self.vertex_w
                        .in_edges_window(t_start.unwrap_or(i64::MIN), t_end.unwrap_or(i64::MAX))
                        .map(|te| te.into()),
                ),
            },
        }
    }

    pub fn out_edges(&self, t_start: Option<i64>, t_end: Option<i64>) -> WindowedEdgeIterator {
        match (t_start, t_end) {
            (None, None) => WindowedEdgeIterator {
                iter: Box::new(self.vertex_w.out_edges().map(|te| te.into())),
            },
            _ => WindowedEdgeIterator {
                iter: Box::new(
                    self.vertex_w
                        .out_edges_window(t_start.unwrap_or(i64::MIN), t_end.unwrap_or(i64::MAX))
                        .map(|te| te.into()),
                ),
            },
        }
    }

    pub fn neighbours(&self, t_start: Option<i64>, t_end: Option<i64>) -> WindowedVertexIterable {
        match (t_start, t_end) {
            (None, None) => WindowedVertexIterable {
                graph: self.graph.clone(),
                operations: vec![Operations::Neighbours],
                start_at: Some(self.id),
            },
            _ => WindowedVertexIterable {
                graph: self.graph.clone(),
                operations: vec![Operations::NeighboursWindow {
                    t_start: t_start.unwrap_or(i64::MIN),
                    t_end: t_end.unwrap_or(i64::MAX),
                }],
                start_at: Some(self.id),
            },
        }
    }

    pub fn in_neighbours(
        &self,
        t_start: Option<i64>,
        t_end: Option<i64>,
    ) -> WindowedVertexIterable {
        match (t_start, t_end) {
            (None, None) => WindowedVertexIterable {
                graph: self.graph.clone(),
                operations: vec![Operations::InNeighbours],
                start_at: Some(self.id),
            },
            _ => WindowedVertexIterable {
                graph: self.graph.clone(),
                operations: vec![Operations::InNeighboursWindow {
                    t_start: t_start.unwrap_or(i64::MIN),
                    t_end: t_end.unwrap_or(i64::MAX),
                }],
                start_at: Some(self.id),
            },
        }
    }

    pub fn out_neighbours(
        &self,
        t_start: Option<i64>,
        t_end: Option<i64>,
    ) -> WindowedVertexIterable {
        match (t_start, t_end) {
            (None, None) => WindowedVertexIterable {
                graph: self.graph.clone(),
                operations: vec![Operations::OutNeighbours],
                start_at: Some(self.id),
            },
            _ => WindowedVertexIterable {
                graph: self.graph.clone(),
                operations: vec![Operations::OutNeighboursWindow {
                    t_start: t_start.unwrap_or(i64::MIN),
                    t_end: t_end.unwrap_or(i64::MAX),
                }],
                start_at: Some(self.id),
            },
        }
    }

    pub fn neighbours_ids(&self, t_start: Option<i64>, t_end: Option<i64>) -> VertexIdsIterator {
        match (t_start, t_end) {
            (None, None) => VertexIdsIterator {
                iter: Box::new(self.vertex_w.neighbours().id()),
            },
            _ => VertexIdsIterator {
                iter: Box::new(
                    self.vertex_w
                        .neighbours_window(t_start.unwrap_or(i64::MIN), t_end.unwrap_or(i64::MAX))
                        .id(),
                ),
            },
        }
    }

    pub fn in_neighbours_ids(&self, t_start: Option<i64>, t_end: Option<i64>) -> VertexIdsIterator {
        match (t_start, t_end) {
            (None, None) => VertexIdsIterator {
                iter: Box::new(self.vertex_w.in_neighbours().id()),
            },
            _ => VertexIdsIterator {
                iter: Box::new(
                    self.vertex_w
                        .in_neighbours_window(
                            t_start.unwrap_or(i64::MIN),
                            t_end.unwrap_or(i64::MAX),
                        )
                        .id(),
                ),
            },
        }
    }

    pub fn out_neighbours_ids(
        &self,
        t_start: Option<i64>,
        t_end: Option<i64>,
    ) -> VertexIdsIterator {
        match (t_start, t_end) {
            (None, None) => VertexIdsIterator {
                iter: Box::new(self.vertex_w.out_neighbours().id()),
            },
            _ => VertexIdsIterator {
                iter: Box::new(
                    self.vertex_w
                        .out_neighbours_window(
                            t_start.unwrap_or(i64::MIN),
                            t_end.unwrap_or(i64::MAX),
                        )
                        .id(),
                ),
            },
        }
    }

    pub fn __repr__(&self) -> String {
        format!("Vertex({})", self.id)
    }
}

#[pyclass]
pub struct WindowedEdge {
    pub(crate) edge_w: graph_window::WindowedEdge,
}

impl From<graph_window::WindowedEdge> for WindowedEdge {
    fn from(value: graph_window::WindowedEdge) -> WindowedEdge {
        WindowedEdge { edge_w: value }
    }
}

#[pymethods]
impl WindowedEdge {

    pub fn has_property(&self,name:String,include_static:Option<bool>) -> bool {
        match include_static {
            None => {self.edge_w.has_property(name,true)}
            Some(b) => {self.edge_w.has_property(name,b)}
        }
    }

    pub fn property(&self,name:String,include_static:Option<bool>) -> Option<Prop> {
        let res = match include_static {
            None => {self.edge_w.property(name,true)}
            Some(b) => {self.edge_w.property(name,b)}
        };

        match res{
            None => {None}
            Some(prop) => {Some(prop.into())}
        }
    }

    pub fn properties(&self,include_static:Option<bool>) -> HashMap<String,Prop> {
        match include_static {
            None => {self.edge_w.properties(true)}
            Some(b) => {self.edge_w.properties(b)}
        }.into_iter()
         .map(|(k, v)| (k, v.into()))
         .collect()
    }

    pub fn property_names(&self,include_static:Option<bool>) -> Vec<String> {
        match include_static {
            None => {self.edge_w.property_names(true)}
            Some(b) => {self.edge_w.property_names(b)}
        }

    }

    pub fn property_history(&self,name:String) -> Vec<(i64, Prop)> {
        self.edge_w.property_history(name).into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect()
    }

    pub fn property_histories(&self) -> HashMap<String, Vec<(i64, Prop)>> {
        self.edge_w.property_histories().into_iter()
            .map(|(k, v)| (k, v.into_iter().map(|(t,p)| (t,p.into())).collect()))
            .collect()
    }

    pub fn has_static_property(&self,name:String)->bool {
        self.edge_w.has_static_property(name)
    }
    pub fn static_property(&self,name:String)-> Option<Prop>{
        match self.edge_w.static_property(name) {
            None => {None}
            Some(prop) => {Some(prop.into())}
        }
    }

    pub fn __getitem__(&self, name: String) -> Option<Prop> {
        self.property(name,Some(true))
    }


    pub fn id(&self) -> usize {
        self.edge_w.id()
    }

    fn src(&self) -> u64 {
        //FIXME can't currently return the WindowedVertex as can't create a Py<WindowedGraph>
        self.edge_w.src().id()
    }

    fn dst(&self) -> u64 {
        //FIXME can't currently return the WindowedVertex as can't create a Py<WindowedGraph>
        self.edge_w.dst().id()
    }
}