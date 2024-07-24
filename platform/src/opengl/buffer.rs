use glow::HasContext;

use super::program::ShaderDataType;

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum BufferType {
    Vertex = glow::ARRAY_BUFFER,
    Index = glow::ELEMENT_ARRAY_BUFFER,
    DrawIndirectCommand = glow::DRAW_INDIRECT_BUFFER,
    ShaderStorage = glow::SHADER_STORAGE_BUFFER,
    Texture = glow::TEXTURE_BUFFER,
    Uniform = glow::UNIFORM_BUFFER,
    TransformFeedback = glow::TRANSFORM_FEEDBACK_BUFFER,
}

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum BufferUsage {
    StaticDraw = glow::STATIC_DRAW,
    StaticRead = glow::STATIC_READ,
    StaticCopy = glow::STATIC_COPY,
    DynamicDraw = glow::DYNAMIC_DRAW,
    DynamicRead = glow::DYNAMIC_READ,
    DynamicCopy = glow::DYNAMIC_COPY,
    StreamDraw = glow::STREAM_DRAW,
    StreamRead = glow::STREAM_READ,
    StreamCopy = glow::STREAM_COPY,
}

/// Describes a single type of data to be used in a gl buffer.\
/// For example, this can be used to desribe data stored in a vertex buffer object that will be presented
/// as input data to a shader.
pub struct BufferElement {
    pub name: &'static str,
    pub data_type: ShaderDataType,
    pub count: u32,
    pub offset: u32,
    pub normalised: bool,
}

impl BufferElement {
    pub fn new(data_type: ShaderDataType, name: &'static str) -> Self {
        let count = data_type.element_count();
        Self {
            name,
            data_type,
            count,
            offset: 0,
            normalised: false,
        }
    }
}

/// The concrete layout of data in a singular gl buffer.\
/// This can be comprised of many [BufferElement] as they will be interleaved into a single buffer.
pub struct BufferLayout {
    pub elements: Vec<BufferElement>,
    pub stride: u32,
    pub buffer_size: u32,
    pub divisor: u32,
    pub usage: BufferUsage,
}

impl BufferLayout {
    pub fn new(
        mut elements: Vec<BufferElement>,
        capacity: u32,
        divisor: u32,
        usage: BufferUsage,
    ) -> Self {
        let mut offset = 0;
        for element in elements.iter_mut() {
            element.offset = offset;
            offset += element.data_type.size_bytes();
        }

        let buffer_size = offset * capacity;

        Self {
            elements,
            stride: offset,
            buffer_size,
            divisor,
            usage,
        }
    }
}

pub struct BufferStorage {
    pub handle: glow::Buffer,
    pub buffer_type: BufferType,
    pub layout: BufferLayout,
}

impl BufferStorage {
    pub fn new(gl: &glow::Context, buffer_type: BufferType, layout: BufferLayout) -> Self {
        let handle = unsafe { gl.create_buffer().expect("Failed to create buffer") };

        unsafe {
            gl.bind_buffer(buffer_type as u32, Some(handle));
            gl.buffer_data_size(
                buffer_type as u32,
                layout.buffer_size as i32,
                layout.usage as u32,
            );
            gl.bind_buffer(buffer_type as u32, None);
        }

        Self {
            handle,
            buffer_type,
            layout,
        }
    }

    /// Expects this [BufferStorage] to already be bound
    pub fn set_data<T>(&mut self, gl: &glow::Context, offset: i32, vertex_data: &[T]) {
        let data: &[u8] = unsafe {
            std::slice::from_raw_parts(
                vertex_data.as_ptr() as *const u8,
                vertex_data.len() * std::mem::size_of::<T>(),
            )
        };

        unsafe { gl.buffer_sub_data_u8_slice(self.buffer_type as u32, offset, data) }
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe { gl.bind_buffer(self.buffer_type as u32, Some(self.handle)) }
    }

    pub fn unbind(gl: &glow::Context, buffer_type: BufferType) {
        unsafe { gl.bind_buffer(buffer_type as u32, None) }
    }

    pub fn dispose(&self, gl: &glow::Context) {
        unsafe { gl.delete_buffer(self.handle) }
    }
}

pub struct VertexArray {
    pub handle: glow::VertexArray,
    pub vertex_buffers: Vec<BufferStorage>,
}

impl VertexArray {
    pub fn new(gl: &glow::Context, layouts: Vec<BufferLayout>) -> Self {
        let vao = unsafe { gl.create_vertex_array().expect("Failed to create vertex array") };

        let buffers = layouts
            .into_iter()
            .map(|layout| BufferStorage::new(gl, BufferType::Vertex, layout))
            .collect();

        let vertex_array = Self {
            handle: vao,
            vertex_buffers: buffers,
        };

        vertex_array.attach_buffers(gl);

        vertex_array
    }

    fn attach_buffers(&self, gl: &glow::Context) {
        self.bind(gl);
        self.attach_vertex_buffers(gl);
        VertexArray::unbind(gl);
        BufferStorage::unbind(gl, BufferType::Vertex);
    }

    /// Expects this [VertexArray] to already be bound
    fn attach_vertex_buffers(&self, gl: &glow::Context) {
        let mut attrib_index = 0;

        for buffer in self.vertex_buffers.iter() {
            unsafe {
                // calling bind buffer here for `vertex_attrib_pointer` ops
                buffer.bind(gl);

                for element in buffer.layout.elements.iter() {
                    gl.enable_vertex_attrib_array(attrib_index);

                    let gl_data_type = element.data_type.gl_type();

                    if gl_data_type == glow::FLOAT {
                        gl.vertex_attrib_pointer_f32(
                            attrib_index,
                            element.count as i32,
                            gl_data_type,
                            element.normalised,
                            buffer.layout.stride as i32,
                            element.offset as i32,
                        );
                    } else {
                        gl.vertex_attrib_pointer_i32(
                            attrib_index,
                            element.count as i32,
                            gl_data_type,
                            buffer.layout.stride as i32,
                            element.offset as i32,
                        );
                    }

                    // Requires > OpenGL 3.3 / OpenGL ES 3.0
                    if buffer.layout.divisor > 0 {
                        gl.vertex_attrib_divisor(attrib_index, buffer.layout.divisor);
                    }

                    attrib_index += 1;
                }
            }
        }
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe { gl.bind_vertex_array(Some(self.handle)) }
    }

    pub fn unbind(gl: &glow::Context) {
        unsafe { gl.bind_vertex_array(None) }
    }

    pub fn dispose(&self, gl: &glow::Context) {
        unsafe { gl.delete_vertex_array(self.handle) }
    }

    pub fn dispose_buffers(&self, gl: &glow::Context) {
        for buffer in self.vertex_buffers.iter() {
            buffer.dispose(gl);
        }
    }
}
