use glow::HasContext;

pub enum ShaderDataType {
    Uint1,
    Uint2,
    Uint3,
    Uint4,
    Int1,
    Int2,
    Int3,
    Int4,
    Float1,
    Float2,
    Float3,
    Float4,
    Mat2f,
    Mat3f,
    Mat4f,
}

impl ShaderDataType {
    pub fn element_count(&self) -> u32 {
        match self {
            ShaderDataType::Uint1 => 1,
            ShaderDataType::Uint2 => 2,
            ShaderDataType::Uint3 => 3,
            ShaderDataType::Uint4 => 4,
            ShaderDataType::Int1 => 1,
            ShaderDataType::Int2 => 2,
            ShaderDataType::Int3 => 3,
            ShaderDataType::Int4 => 4,
            ShaderDataType::Float1 => 1,
            ShaderDataType::Float2 => 2,
            ShaderDataType::Float3 => 3,
            ShaderDataType::Float4 => 4,
            ShaderDataType::Mat2f => 4,
            ShaderDataType::Mat3f => 9,
            ShaderDataType::Mat4f => 16,
        }
    }

    pub fn size_bytes(&self) -> u32 {
        match self {
            ShaderDataType::Uint1 => 1 * 4,
            ShaderDataType::Uint2 => 2 * 4,
            ShaderDataType::Uint3 => 3 * 4,
            ShaderDataType::Uint4 => 4 * 4,
            ShaderDataType::Int1 => 1 * 4,
            ShaderDataType::Int2 => 2 * 4,
            ShaderDataType::Int3 => 3 * 4,
            ShaderDataType::Int4 => 4 * 4,
            ShaderDataType::Float1 => 1 * 4,
            ShaderDataType::Float2 => 2 * 4,
            ShaderDataType::Float3 => 3 * 4,
            ShaderDataType::Float4 => 4 * 4,
            ShaderDataType::Mat2f => 4 * 4,
            ShaderDataType::Mat3f => 9 * 4,
            ShaderDataType::Mat4f => 16 * 4,
        }
    }

    pub fn gl_type(&self) -> u32 {
        match self {
            ShaderDataType::Uint1
            | ShaderDataType::Uint2
            | ShaderDataType::Uint3
            | ShaderDataType::Uint4 => glow::UNSIGNED_INT,
            ShaderDataType::Int1
            | ShaderDataType::Int2
            | ShaderDataType::Int3
            | ShaderDataType::Int4 => glow::INT,
            ShaderDataType::Float1
            | ShaderDataType::Float2
            | ShaderDataType::Float3
            | ShaderDataType::Float4
            | ShaderDataType::Mat2f
            | ShaderDataType::Mat3f
            | ShaderDataType::Mat4f => glow::FLOAT,
        }
    }
}

pub struct ShaderSource<'a> {
    vertex: &'a str,
    fragment: &'a str,
}

impl<'a> ShaderSource<'a> {
    pub fn new(vertex: &'a str, fragment: &'a str) -> Self {
        Self { vertex, fragment }
    }
}

pub struct Program {
    pub handle: glow::Program,
    pub shader_handles: Vec<glow::Shader>,
}

impl Program {
    pub fn new(gl: &glow::Context, shaders: ShaderSource) -> Self {
        let gl_prog = unsafe { gl.create_program().expect("Failed to create program") };

        let mut program = Self {
            handle: gl_prog,
            shader_handles: Vec::new(),
        };

        println!("Adding vertex shader");
        program.add_shader(gl, glow::VERTEX_SHADER, &shaders.vertex);
        println!("Adding fragment shader");
        program.add_shader(gl, glow::FRAGMENT_SHADER, &shaders.fragment);

        program
    }

    fn add_shader(&mut self, gl: &glow::Context, shader_type: u32, shader_source: &str) {
        let shader = unsafe {
            gl.create_shader(shader_type)
                .expect("Failed to create shader")
        };

        unsafe {
            gl.shader_source(shader, shader_source);
        }

        self.compile_shader(gl, shader);
        self.link_shader(gl, shader);

        self.shader_handles.push(shader);
    }

    fn compile_shader(&self, gl: &glow::Context, shader: glow::Shader) {
        unsafe {
            // Compiles the source code strings that have been stored in the shader object
            gl.compile_shader(shader);
        }

        self.print_shader_compile_status(gl, shader);
        self.print_shader_info_log(gl, shader);
    }

    fn link_shader(&self, gl: &glow::Context, shader: glow::Shader) {
        unsafe {
            // We associate the shader object with a source code string
            gl.attach_shader(self.handle, shader);

            // This uses the attached shader objects to create a single executable to run on the GPU
            gl.link_program(self.handle);

            // If a shader object to be deleted is attached to a program object, it will be flagged for deletion, but
            // it will not be deleted until it is no longer attached to any program object
            gl.delete_shader(shader);
        }

        self.print_program_link_status(gl);
        self.print_program_info_log(gl);
    }

    /// Prints the information log for the specified shader object.\
    /// The information log for a shader object is modified when the shader is compiled.
    fn print_shader_info_log(&self, gl: &glow::Context, shader: glow::Shader) {
        let msg = unsafe { gl.get_shader_info_log(shader) };
        let msg = msg.trim();
        if msg.is_empty() {
            return;
        }

        println!("Shader Info Log: \n{}", msg);
    }

    fn print_shader_compile_status(&self, gl: &glow::Context, shader: glow::Shader) {
        println!("Shader Compile Status: {}", unsafe {
            gl.get_shader_compile_status(shader)
        })
    }

    fn print_program_info_log(&self, gl: &glow::Context) {
        let msg = unsafe { gl.get_program_info_log(self.handle) };
        let msg = msg.trim();
        if msg.is_empty() {
            return;
        }

        println!("Program Info Log: \n{}", msg);
    }

    fn print_program_link_status(&self, gl: &glow::Context) {
        println!("Program Link Status: {}", unsafe {
            gl.get_program_link_status(self.handle)
        });
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.use_program(Some(self.handle));
        }
    }

    pub fn unbind(gl: &glow::Context) {
        unsafe {
            gl.use_program(None);
        }
    }

    pub fn dispose(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.handle);
        }
    }
}
