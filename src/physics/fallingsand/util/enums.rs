/// The different ways to draw a mesh
#[derive(Copy, Clone, PartialEq)]
pub enum MeshDrawMode {
    TexturedMesh,
    Outline,
    TriangleWireframe,
}

/// How to draw the mesh to, in the future, be handled by zoom levels
#[derive(Copy, Clone, PartialEq)]
pub enum ZoomDrawMode {
    FrustumCull,
    Combine,
}
