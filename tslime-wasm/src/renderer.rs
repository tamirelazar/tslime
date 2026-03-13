use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{
    WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlTexture, WebGlUniformLocation,
};

const VERTEX_SHADER_SOURCE: &str = r#"#version 300 es
    in vec2 a_position;
    out vec2 v_texCoord;

    void main() {
        v_texCoord = a_position * 0.5 + 0.5;
        gl_Position = vec4(a_position, 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"#version 300 es
    precision highp float;

    in vec2 v_texCoord;
    out vec4 outColor;

    uniform sampler2D u_trailTexture;
    uniform float u_brightness;
    uniform vec3 u_color;

    void main() {
        float trail = texture(u_trailTexture, v_texCoord).r;
        trail = clamp(trail * u_brightness, 0.0, 1.0);
        outColor = vec4(u_color * trail, 1.0);
    }
"#;

pub struct WebGlRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    trail_texture: WebGlTexture,
    quad_vao: web_sys::WebGlVertexArrayObject,
    trail_texture_location: WebGlUniformLocation,
    brightness_location: WebGlUniformLocation,
    color_location: WebGlUniformLocation,
    width: u32,
    height: u32,
}

impl WebGlRenderer {
    pub fn new(canvas_id: &str) -> Result<Rc<RefCell<Self>>, JsValue> {
        let window = web_sys::window().ok_or("no window")?;
        let document = window.document().ok_or("no document")?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or("canvas not found")?;
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into()?;

        let context = canvas
            .get_context("webgl2")?
            .ok_or("webgl2 not supported")?
            .dyn_into::<WebGl2RenderingContext>()?;

        let width = canvas.width() as u32;
        let height = canvas.height() as u32;

        let program = Self::create_program(&context)?;
        let trail_texture = Self::create_texture(&context, width, height)?;
        let quad_vao = Self::create_quad(&context, &program)?;

        let trail_texture_location = context
            .get_uniform_location(&program, "u_trailTexture")
            .ok_or("u_trailTexture location not found")?;
        let brightness_location = context
            .get_uniform_location(&program, "u_brightness")
            .ok_or("u_brightness location not found")?;
        let color_location = context
            .get_uniform_location(&program, "u_color")
            .ok_or("u_color location not found")?;

        context.use_program(Some(&program));

        Ok(Rc::new(RefCell::new(Self {
            context,
            program,
            trail_texture,
            quad_vao,
            trail_texture_location,
            brightness_location,
            color_location,
            width,
            height,
        })))
    }

    fn create_shader(
        context: &WebGl2RenderingContext,
        shader_type: u32,
        source: &str,
    ) -> Result<WebGlShader, JsValue> {
        let shader = context
            .create_shader(shader_type)
            .ok_or("Unable to create shader")?;
        context.shader_source(&shader, source);
        context.compile_shader(&shader);

        if context
            .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            let info = context.get_shader_info_log(&shader).unwrap_or_default();
            context.delete_shader(Some(&shader));
            Err(JsValue::from_str(&format!(
                "Shader compile error: {}",
                info
            )))
        }
    }

    fn create_program(context: &WebGl2RenderingContext) -> Result<WebGlProgram, JsValue> {
        let vertex_shader = Self::create_shader(
            context,
            WebGl2RenderingContext::VERTEX_SHADER,
            VERTEX_SHADER_SOURCE,
        )?;
        let fragment_shader = Self::create_shader(
            context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            FRAGMENT_SHADER_SOURCE,
        )?;

        let program = context.create_program().ok_or("Unable to create program")?;
        context.attach_shader(&program, &vertex_shader);
        context.attach_shader(&program, &fragment_shader);
        context.link_program(&program);

        if context
            .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(program)
        } else {
            let info = context.get_program_info_log(&program).unwrap_or_default();
            context.delete_program(Some(&program));
            Err(JsValue::from_str(&format!("Program link error: {}", info)))
        }
    }

    fn create_texture(
        context: &WebGl2RenderingContext,
        width: u32,
        height: u32,
    ) -> Result<WebGlTexture, JsValue> {
        let texture = context.create_texture().ok_or("Unable to create texture")?;

        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );

        let empty_data = vec![0u8; (width * height * 4) as usize];
        context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA8 as i32,
            width as i32,
            height as i32,
            0,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(&empty_data),
        )?;

        Ok(texture)
    }

    fn create_quad(
        context: &WebGl2RenderingContext,
        program: &WebGlProgram,
    ) -> Result<web_sys::WebGlVertexArrayObject, JsValue> {
        let vao = context
            .create_vertex_array()
            .ok_or("Unable to create VAO")?;
        context.bind_vertex_array(Some(&vao));

        let positions: [f32; 12] = [
            -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0,
        ];

        let position_buffer = context
            .create_buffer()
            .ok_or("Unable to create position buffer")?;
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&position_buffer));
        context.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            unsafe {
                std::slice::from_raw_parts(positions.as_ptr() as *const u8, positions.len() * 4)
            },
            WebGl2RenderingContext::STATIC_DRAW,
        );

        let position_location = context.get_attrib_location(program, "a_position");
        context.enable_vertex_attrib_array(position_location as u32);
        context.vertex_attrib_pointer_with_i32(
            position_location as u32,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );

        context.bind_vertex_array(None);

        Ok(vao)
    }

    pub fn update_texture(&self, trail_data: &[f32]) {
        let size = (self.width * self.height) as usize;
        let mut rgba_data = vec![0u8; size * 4];

        for i in 0..size {
            let value = if i < trail_data.len() {
                (trail_data[i].clamp(0.0, 1.0) * 255.0) as u8
            } else {
                0
            };
            rgba_data[i * 4] = value;
            rgba_data[i * 4 + 1] = value;
            rgba_data[i * 4 + 2] = value;
            rgba_data[i * 4 + 3] = 255;
        }

        self.context.bind_texture(
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&self.trail_texture),
        );
        self.context
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                0,
                0,
                self.width as i32,
                self.height as i32,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                Some(&rgba_data),
            )
            .ok();
    }

    pub fn render(&self, brightness: f32, color: (f32, f32, f32)) {
        self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.context.use_program(Some(&self.program));
        self.context.bind_vertex_array(Some(&self.quad_vao));

        self.context
            .active_texture(WebGl2RenderingContext::TEXTURE0);
        self.context.bind_texture(
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&self.trail_texture),
        );
        self.context
            .uniform1i(Some(&self.trail_texture_location), 0);
        self.context
            .uniform1f(Some(&self.brightness_location), brightness);
        self.context
            .uniform3f(Some(&self.color_location), color.0, color.1, color.2);

        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), JsValue> {
        self.width = width;
        self.height = height;

        self.trail_texture = Self::create_texture(&self.context, width, height)?;

        self.context.viewport(0, 0, width as i32, height as i32);

        Ok(())
    }
}
