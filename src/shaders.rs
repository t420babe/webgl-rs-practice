use super::*;
use crate::{
  buffer_attrib, buffer_attrib::BufferAttrib, buffers, program_info::ProgramInfo, utils::*,
};
use nalgebra_glm;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
  console, AudioContext, EventTarget, HtmlCanvasElement, HtmlMediaElement, WebGl2RenderingContext,
  WebGlBuffer,
};

pub fn draw_scene(
  gl_context: &WebGl2RenderingContext,
  program_info: ProgramInfo,
  buffers: HashMap<String, WebGlBuffer>,
  time: f32,
) -> Result<(), JsValue> {
  gl_context.clear_color(1.0, 0.5, 0.5, 1.0);
  // gl_context.clear_depth(0.0);
  gl_context.enable(WebGl2RenderingContext::DEPTH_TEST);
  gl_context.depth_func(WebGl2RenderingContext::LEQUAL); // Near objects obscure far ones

  // Clear the canvas before drawing to it
  gl_context
    .clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);

  // Projection and model view matrices
  let projection_matrix = create_perspective_matrix(&gl_context)?;
  let model_view_matrix = create_model_view_matrix(time);

  // Tell WebGl to pull out the positions from the vertices buffer into the `a_vertex_position` attribute
  let a_vertex_position = (*program_info
    .attrib_locations
    .get(&"a_vertex_position".to_string())
    .ok_or("Failed to get `a_vertex_position` attribute")?) as u32;

  let a_vertex_position_buffer_attrib = BufferAttrib {
    name: "vertices".into(),
    buffer: buffers
      .get(&"vertices".to_string())
      .ok_or("Failed to get `a_vertex_position` attribute")?,
    target: WebGl2RenderingContext::ARRAY_BUFFER,
    num_components: 2,
    buffer_type: WebGl2RenderingContext::FLOAT,
    normalize: false,
    stride: 0,
    offset: 0,
  };
  buffer_attrib::bind_buffer_to_attrib(
    &gl_context,
    &a_vertex_position_buffer_attrib,
    a_vertex_position,
  )?;

  let a_vertex_color = (*program_info
    .attrib_locations
    .get(&"a_vertex_color".to_string())
    .ok_or("Failed to get `a_vertex_color` attribute")?) as u32;
  let a_vertex_color_buffer_attrib = BufferAttrib {
    name: "colors".into(),
    buffer: buffers.get(&"colors".to_string()).ok_or("Failed to get `a_vertex_color` attribute")?,
    target: WebGl2RenderingContext::ARRAY_BUFFER,
    num_components: 4,
    buffer_type: WebGl2RenderingContext::FLOAT,
    normalize: false,
    stride: 0,
    offset: 0,
  };
  buffer_attrib::bind_buffer_to_attrib(&gl_context, &a_vertex_color_buffer_attrib, a_vertex_color)?;

  // Tell WebGl to use our program when drawing
  gl_context.use_program(Some(&program_info.program));

  let projection_matrix = &projection_matrix[0..];
  gl_context.uniform_matrix4fv_with_f32_array(
    program_info.uniform_locations.get(&"u_projection_matrix".to_string()).unwrap().as_ref(),
    false,
    projection_matrix,
  );

  let model_view_matrix = &model_view_matrix[0..];
  gl_context.uniform_matrix4fv_with_f32_array(
    program_info.uniform_locations.get(&"u_model_view_matrix".to_string()).unwrap().as_ref(),
    false,
    model_view_matrix,
  );

  gl_context
    .uniform1f(program_info.uniform_locations.get(&"u_time".to_string()).unwrap().as_ref(), time);

  let vertex_count = 4;
  let offset = 0; // How many bytes inside the buffer to start from
  gl_context.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, offset, vertex_count);

  Ok(())
}

fn create_perspective_matrix(gl_context: &WebGl2RenderingContext) -> Result<[f32; 16], JsValue> {
  // Create a perspective matrix, a special matrix that is used to simulate the distortion of perspective in a camera.
  // Our field of view is 45 degrees, which a width/height ratio that matches the display size of the canvas and we
  // only want to see objects between 0.1 and 100.0 units away from the camera
  let field_of_view = 45.0 * std::f32::consts::PI / 180.0;
  let canvas: HtmlCanvasElement = gl_context
    .canvas()
    .ok_or("Failed to get canvas on draw")?
    .dyn_into::<web_sys::HtmlCanvasElement>()?;
  let aspect = (canvas.client_width() / canvas.client_height()) as f32;
  let z_near = 0.1;
  let z_far = 100.0;
  let projection_matrix = nalgebra_glm::perspective(aspect, field_of_view, z_near, z_far);
  Ok(mat4_to_f32_16(projection_matrix))
}

/// Rotate the square
fn create_model_view_matrix(angle: f32) -> [f32; 16] {
  let model_view_matrix = nalgebra_glm::identity();
  let translation_vector = nalgebra_glm::vec3(0.0, 0.0, -6.0);
  let translated_matrix = nalgebra_glm::translate(&model_view_matrix, &translation_vector);
  let rotation_vector = nalgebra_glm::vec3(0.0, 0.0, 1.0);
  let rotated_matrix = nalgebra_glm::rotate(&translated_matrix, angle, &rotation_vector);
  mat4_to_f32_16(rotated_matrix)
}

pub(crate) fn do_webgl(gl_context: WebGl2RenderingContext) -> Result<(), JsValue> {
  /* WebGl */

  let program_info = ProgramInfo::new(&gl_context)?;

  let buffers = buffers::make_buffers(&gl_context)?;

  // Draw scene every 0.01 seconds
  let ref_count = Rc::new(RefCell::new(None));
  let ref_count_clone = ref_count.clone();

  *ref_count_clone.borrow_mut() = Some(Closure::wrap(Box::new(move |t| {
    draw_scene(&gl_context.clone(), program_info.clone(), buffers.clone(), t * 0.001f32).unwrap();
    request_animation_frame(ref_count.borrow().as_ref().unwrap());
  }) as Box<dyn FnMut(f32)>));

  request_animation_frame(ref_count_clone.borrow().as_ref().unwrap());

  Ok(())
}
