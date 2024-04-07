struct VertexOutput {
    @builtin(position) clip_pos: vec4f,
    @location(0) color: vec4f,
};

@vertex
fn vert(
    @builtin(vertex_index) i: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(i)) * 0.5;
    let y = f32(i32(i & 1u) * 2 - 1) * 0.5;
    out.clip_pos = vec4f(x, y, 0.0, 1.0);
    if i == 0 {
        out.color = vec4f(0.9, 0.3, 0.6, 1.0);
    } else if i == 1 {
        out.color = vec4f(0.9, 0.9, 0.9, 1.0);
    } else {
        out.color = vec4f(0.3, 0.5, 0.9, 1.0);
    }
    return out;
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    return in.color;
}
