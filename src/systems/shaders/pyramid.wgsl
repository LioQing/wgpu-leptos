struct Camera {
    view_projection: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

struct PyramidTransform {
    transform: mat4x4<f32>,
}
@group(1) @binding(0)
var<uniform> pyramid_transform: PyramidTransform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = camera.view_projection * pyramid_transform.transform * vec4<f32>(input.position, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}