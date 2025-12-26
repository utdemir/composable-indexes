#![allow(dead_code)]

use composable_indexes::index;

#[derive(PartialEq, Eq, PartialOrd, Hash, Ord, Clone, Copy)]
struct VertexId(u32);

#[derive(PartialEq, Eq, PartialOrd, Hash, Ord, Clone)]
struct VertexPayload(String);

struct Vertex {
    id: VertexId,
    payload: VertexPayload,
}

struct Edge {
    from: VertexId,
    to: VertexId,
    weight: u64,
}

type VertexIndex = index::zip::ZipIndex2<
    Vertex,
    index::PremapOwnedIndex<Vertex, VertexId, index::HashTableIndex<VertexId>>,
    index::PremapIndex<Vertex, VertexPayload, index::HashTableIndex<VertexPayload>>,
>;

type EdgeIndex = index::zip::ZipIndex4<
    Edge,
    index::PremapOwnedIndex<
        Edge,
        (VertexId, VertexId),
        index::HashTableIndex<(VertexId, VertexId)>,
    >,
    index::GroupedIndex<
        Edge,
        VertexId,
        index::PremapIndex<Edge, VertexId, index::HashTableIndex<VertexId>>,
    >,
    index::GroupedIndex<
        Edge,
        VertexId,
        index::PremapIndex<Edge, VertexId, index::HashTableIndex<VertexId>>,
    >,
    index::PremapIndex<Edge, u64, index::BTreeIndex<u64>>,
>;

struct Graph {
    vertices: composable_indexes::Collection<Vertex, VertexIndex>,
    edges: composable_indexes::Collection<Edge, EdgeIndex>,
}

impl Graph {
    fn new() -> Self {
        Self {
            vertices: composable_indexes::Collection::<Vertex, VertexIndex>::new(index::zip!(
                index::PremapOwnedIndex::new(|v: &Vertex| v.id, index::HashTableIndex::new()),
                index::PremapIndex::new(|v: &Vertex| &v.payload, index::HashTableIndex::new()),
            )),
            edges: composable_indexes::Collection::<Edge, EdgeIndex>::new(index::zip!(
                index::PremapOwnedIndex::new(
                    |e: &Edge| (e.from, e.to),
                    index::HashTableIndex::new()
                ),
                index::GroupedIndex::new(
                    |e: &Edge| &e.from,
                    || index::PremapIndex::new(|e: &Edge| &e.to, index::HashTableIndex::new())
                ),
                index::GroupedIndex::new(
                    |e: &Edge| &e.to,
                    || index::PremapIndex::new(|e: &Edge| &e.from, index::HashTableIndex::new())
                ),
                index::PremapIndex::new(|e: &Edge| &e.weight, index::BTreeIndex::<u64>::new()),
            )),
        }
    }

    fn add_vertex(&mut self, id: VertexId, payload: VertexPayload) {
        let vertex = Vertex { id, payload };
        self.vertices.insert(vertex);
    }

    fn remove_vertex(&mut self, id: &VertexId) {
        self.vertices.delete(|ix| ix._1().get_one(id));
        self.edges
            .delete(|ix| (ix._2().get(id).all(), ix._3().get(id).all()));
        self.edges.delete(|ix| ix._3().get(id).all());
    }

    fn connect(&mut self, from: VertexId, to: VertexId, weight: u64) {
        let edge = Edge { from, to, weight };
        self.disconnect(from, to);
        self.edges.insert(edge);
    }

    fn disconnect(&mut self, from: VertexId, to: VertexId) {
        self.edges.delete(|ix| ix._1().get_one(&(from, to)));
    }

    fn downstream(&self, vertex_id: &VertexId) -> Vec<&Edge> {
        self.edges.query(|ix| ix._2().get(vertex_id).all())
    }

    fn upstream(&self, vertex_id: &VertexId) -> Vec<&Edge> {
        self.edges.query(|ix| ix._3().get(vertex_id).all())
    }
}

fn main() {
    //
}
