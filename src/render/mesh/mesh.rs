use crate::ops::{Bindable, Drawable};
use crate::utils::Identifiable;

use gl::types::{GLint, GLsizeiptr, GLuint};
use math::vector::Vector3;
use std::mem;
use std::ptr;
use std::vec::Vec;

#[derive(Debug)]
pub struct Mesh {
    vao: GLuint,
    vbo_count: GLuint,
    index_count: usize,
    buffers: Vec<GLuint>,
}

impl Mesh {
    pub fn new(vertices: &Vec<Vector3>, indices: &Vec<GLuint>) -> Self {
        let mut mesh = Self {
            vao: 0,
            vbo_count: 0,
            index_count: indices.len(),
            buffers: Vec::new(),
        };

        unsafe {
            gl::GenVertexArrays(1, &mut mesh.vao);
        }

        mesh.add_vbo(vertices);
        mesh.add_ebo(indices);
        mesh
    }

    pub fn add_vbo<T>(&mut self, data: &Vec<T>) {
        let mut vbo: GLuint = 0;

        self.bind();

        unsafe {
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                mem::transmute(&data[0]),
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(
                self.vbo_count,
                (mem::size_of::<T>() / 4) as i32,
                gl::FLOAT,
                gl::FALSE,
                0,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(self.vbo_count);
        }

        self.buffers.push(vbo);
        self.vbo_count += 1;
    }

    pub fn add_vbo_u32(&mut self, data: &Vec<u32>) {
        let mut vbo: GLuint = 0;

        self.bind();

        unsafe {
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * mem::size_of::<u32>()) as GLsizeiptr,
                data.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribIPointer(
                self.vbo_count,
                1,
                gl::UNSIGNED_INT,
                0,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(self.vbo_count);
        }

        self.buffers.push(vbo);
        self.vbo_count += 1;
    }


    fn add_ebo(&mut self, indices: &Vec<GLuint>) {
        let mut ebo: GLuint = 0;

        unsafe {
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                mem::transmute(&indices[0]),
                gl::STATIC_DRAW,
            );
        }

        self.buffers.push(ebo);
    }
}

impl Identifiable for Mesh {
    type Id = gl::types::GLuint;

    fn id(&self) -> Self::Id {
        self.vao
    }
}

impl Drawable for Mesh {
    fn draw(&self) {
        self.bind();

        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count as GLint,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }

        self.unbind();
    }
}

impl Bindable for Mesh {
    fn bind(&self) {
        unsafe { gl::BindVertexArray(self.vao) }
    }

    fn unbind(&self) {
        unsafe { gl::BindVertexArray(0) }
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(self.buffers.len() as i32, self.buffers.as_mut_ptr());
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
