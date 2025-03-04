use std::mem;

use gl::INT_VEC3;

use super::Attribute;

// From https://github.com/glium/glium/blob/master/src/vertex/format.rs

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AttributeType {
    I8,
    I8I8,
    I8I8I8,
    I8I8I8I8,
    U8,
    U8U8,
    U8U8U8,
    U8U8U8U8,
    I16,
    I16I16,
    I16I16I16,
    I16I16I16I16,
    U16,
    U16U16,
    U16U16U16,
    U16U16U16U16,
    I32,
    I32I32,
    I32I32I32,
    I32I32I32I32,
    U32,
    U32U32,
    U32U32U32,
    U32U32U32U32,
    I64,
    I64I64,
    I64I64I64,
    I64I64I64I64,
    U64,
    U64U64,
    U64U64U64,
    U64U64U64U64,
    F16,
    F16F16,
    F16F16F16,
    F16F16F16F16,
    /// 2x2 matrix of `f16`s
    F16x2x2,
    /// 2x3 matrix of `f16`s
    F16x2x3,
    /// 2x3 matrix of `f16`s
    F16x2x4,
    /// 3x2 matrix of `f16`s
    F16x3x2,
    /// 3x3 matrix of `f16`s
    F16x3x3,
    /// 3x4 matrix of `f16`s
    F16x3x4,
    /// 4x2 matrix of `f16`s
    F16x4x2,
    /// 4x3 matrix of `f16`s
    F16x4x3,
    /// 4x4 matrix of `f16`s
    F16x4x4,
    F32,
    F32F32,
    F32F32F32,
    F32F32F32F32,
    /// 2x2 matrix of `f32`s
    F32x2x2,
    /// 2x3 matrix of `f32`s
    F32x2x3,
    /// 2x3 matrix of `f32`s
    F32x2x4,
    /// 3x2 matrix of `f32`s
    F32x3x2,
    /// 3x3 matrix of `f32`s
    F32x3x3,
    /// 3x4 matrix of `f32`s
    F32x3x4,
    /// 4x2 matrix of `f32`s
    F32x4x2,
    /// 4x3 matrix of `f32`s
    F32x4x3,
    /// 4x4 matrix of `f32`s
    F32x4x4,
    /// Warning: using `f64`s can be very slow.
    F64,
    /// Warning: using `f64`s can be very slow.
    F64F64,
    /// Warning: using `f64`s can be very slow.
    F64F64F64,
    /// Warning: using `f64`s can be very slow.
    F64F64F64F64,
    /// 2x2 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x2x2,
    /// 2x3 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x2x3,
    /// 2x3 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x2x4,
    /// 3x2 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x3x2,
    /// 3x3 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x3x3,
    /// 3x4 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x3x4,
    /// 4x2 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x4x2,
    /// 4x3 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x4x3,
    /// 4x4 matrix of `f64`s
    /// Warning: using `f64`s can be very slow.
    F64x4x4,
    /// From MSB to LSB: two bits for the alpha, ten bits for the blue, ten bits for the green,
    /// ten bits for the red.
    ///
    /// Corresponds to `GL_INT_2_10_10_10_REV`.
    I2I10I10I10Reversed,
    /// From MSB to LSB: two bits for the alpha, ten bits for the blue, ten bits for the green,
    /// ten bits for the red.
    ///
    /// Corresponds to `GL_UNSIGNED_INT_2_10_10_10_REV`.
    U2U10U10U10Reversed,
    /// Corresponds to `GL_INT_10_10_10_2`.
    I10I10I10I2,
    /// Corresponds to `GL_UNSIGNED_INT_10_10_10_2`.
    U10U10U10U2,
    /// Three floating points values turned into unsigned integers./
    ///
    /// Corresponds to `GL_UNSIGNED_INT_10F_11F_11F_REV`.
    F10F11F11UnsignedIntReversed,
    /// Fixed floating points. A 16bits signed value followed by the 16bits unsigned exponent.
    ///
    /// Corresponds to `GL_FIXED`.
    FixedFloatI16U16,
}

impl AttributeType {
    /// Returns the size in bytes of a value of this type.
    pub fn get_size_bytes(&self) -> usize {
        match *self {
            AttributeType::I8 => mem::size_of::<i8>(),
            AttributeType::I8I8 => 2 * mem::size_of::<i8>(),
            AttributeType::I8I8I8 => 3 * mem::size_of::<i8>(),
            AttributeType::I8I8I8I8 => 4 * mem::size_of::<i8>(),
            AttributeType::U8 => mem::size_of::<u8>(),
            AttributeType::U8U8 => 2 * mem::size_of::<u8>(),
            AttributeType::U8U8U8 => 3 * mem::size_of::<u8>(),
            AttributeType::U8U8U8U8 => 4 * mem::size_of::<u8>(),
            AttributeType::I16 => mem::size_of::<i16>(),
            AttributeType::I16I16 => 2 * mem::size_of::<i16>(),
            AttributeType::I16I16I16 => 3 * mem::size_of::<i16>(),
            AttributeType::I16I16I16I16 => 4 * mem::size_of::<i16>(),
            AttributeType::U16 => mem::size_of::<u16>(),
            AttributeType::U16U16 => 2 * mem::size_of::<u16>(),
            AttributeType::U16U16U16 => 3 * mem::size_of::<u16>(),
            AttributeType::U16U16U16U16 => 4 * mem::size_of::<u16>(),
            AttributeType::I32 => mem::size_of::<i32>(),
            AttributeType::I32I32 => 2 * mem::size_of::<i32>(),
            AttributeType::I32I32I32 => 3 * mem::size_of::<i32>(),
            AttributeType::I32I32I32I32 => 4 * mem::size_of::<i32>(),
            AttributeType::U32 => mem::size_of::<u32>(),
            AttributeType::U32U32 => 2 * mem::size_of::<u32>(),
            AttributeType::U32U32U32 => 3 * mem::size_of::<u32>(),
            AttributeType::U32U32U32U32 => 4 * mem::size_of::<u32>(),
            AttributeType::I64 => mem::size_of::<i64>(),
            AttributeType::I64I64 => 2 * mem::size_of::<i64>(),
            AttributeType::I64I64I64 => 3 * mem::size_of::<i64>(),
            AttributeType::I64I64I64I64 => 4 * mem::size_of::<i64>(),
            AttributeType::U64 => mem::size_of::<u64>(),
            AttributeType::U64U64 => 2 * mem::size_of::<u64>(),
            AttributeType::U64U64U64 => 3 * mem::size_of::<u64>(),
            AttributeType::U64U64U64U64 => 4 * mem::size_of::<u64>(),
            AttributeType::F16 => 2,
            AttributeType::F16F16 => 2 * 2,
            AttributeType::F16F16F16 => 3 * 2,
            AttributeType::F16F16F16F16 => 4 * 2,
            AttributeType::F16x2x2 => 4 * 2,
            AttributeType::F16x2x3 => 6 * 2,
            AttributeType::F16x2x4 => 8 * 2,
            AttributeType::F16x3x2 => 6 * 2,
            AttributeType::F16x3x3 => 9 * 2,
            AttributeType::F16x3x4 => 12 * 2,
            AttributeType::F16x4x2 => 8 * 2,
            AttributeType::F16x4x3 => 12 * 2,
            AttributeType::F16x4x4 => 16 * 2,
            AttributeType::F32 => mem::size_of::<f32>(),
            AttributeType::F32F32 => 2 * mem::size_of::<f32>(),
            AttributeType::F32F32F32 => 3 * mem::size_of::<f32>(),
            AttributeType::F32F32F32F32 => 4 * mem::size_of::<f32>(),
            AttributeType::F32x2x2 => 4 * mem::size_of::<f32>(),
            AttributeType::F32x2x3 => 6 * mem::size_of::<f32>(),
            AttributeType::F32x2x4 => 8 * mem::size_of::<f32>(),
            AttributeType::F32x3x2 => 6 * mem::size_of::<f32>(),
            AttributeType::F32x3x3 => 9 * mem::size_of::<f32>(),
            AttributeType::F32x3x4 => 12 * mem::size_of::<f32>(),
            AttributeType::F32x4x2 => 8 * mem::size_of::<f32>(),
            AttributeType::F32x4x3 => 12 * mem::size_of::<f32>(),
            AttributeType::F32x4x4 => 16 * mem::size_of::<f32>(),
            AttributeType::F64 => mem::size_of::<f64>(),
            AttributeType::F64F64 => 2 * mem::size_of::<f64>(),
            AttributeType::F64F64F64 => 3 * mem::size_of::<f64>(),
            AttributeType::F64F64F64F64 => 4 * mem::size_of::<f64>(),
            AttributeType::F64x2x2 => 4 * mem::size_of::<f64>(),
            AttributeType::F64x2x3 => 6 * mem::size_of::<f64>(),
            AttributeType::F64x2x4 => 8 * mem::size_of::<f64>(),
            AttributeType::F64x3x2 => 6 * mem::size_of::<f64>(),
            AttributeType::F64x3x3 => 9 * mem::size_of::<f64>(),
            AttributeType::F64x3x4 => 12 * mem::size_of::<f64>(),
            AttributeType::F64x4x2 => 8 * mem::size_of::<f64>(),
            AttributeType::F64x4x3 => 12 * mem::size_of::<f64>(),
            AttributeType::F64x4x4 => 16 * mem::size_of::<f64>(),
            AttributeType::I2I10I10I10Reversed => 4,
            AttributeType::U2U10U10U10Reversed => 4,
            AttributeType::I10I10I10I2 => 4,
            AttributeType::U10U10U10U2 => 4,
            AttributeType::F10F11F11UnsignedIntReversed => 4,
            AttributeType::FixedFloatI16U16 => 4,
        }
    }

    /// Returns the number of values for this type.
    pub fn get_gl_type(&self) -> u32 {
        use AttributeType::*;
        match *self {
            I32 => gl::INT,
            U32 => gl::UNSIGNED_INT,
            F32 => gl::FLOAT,
            F32F32 => gl::FLOAT_VEC2,
            I32I32 => gl::INT_VEC2,
            F32F32F32 => gl::FLOAT_VEC3,
            I32I32I32 => gl::INT_VEC3,
            F32F32F32F32 => gl::FLOAT_VEC4,
            I32I32I32I32 => gl::INT_VEC4,
            _ => {
                todo!("TODO: Convert {:?} to OpenGL type", self)
            }
        }
    }

    pub fn get_gl_primative(&self) -> u32 {
        use AttributeType::*;
        match *self {
            I32 | I32I32 | I32I32I32 | I32I32I32I32 => gl::INT,
            U32 => gl::UNSIGNED_INT,
            F32 | F32F32 | F32F32F32 | F32F32F32F32 => gl::FLOAT,
            _ => {
                todo!("Convert {:?} to OpenGL primitive", self);
            }
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.get_gl_primative(), gl::INT | gl::UNSIGNED_INT)
    }

    /// Returns the enum for the type in OpenGL
    pub fn get_num_components(&self) -> gl::types::GLenum {
        match *self {
            AttributeType::I8 => 1,
            AttributeType::I8I8 => 2,
            AttributeType::I8I8I8 => 3,
            AttributeType::I8I8I8I8 => 4,
            AttributeType::U8 => 1,
            AttributeType::U8U8 => 2,
            AttributeType::U8U8U8 => 3,
            AttributeType::U8U8U8U8 => 4,
            AttributeType::I16 => 1,
            AttributeType::I16I16 => 2,
            AttributeType::I16I16I16 => 3,
            AttributeType::I16I16I16I16 => 4,
            AttributeType::U16 => 1,
            AttributeType::U16U16 => 2,
            AttributeType::U16U16U16 => 3,
            AttributeType::U16U16U16U16 => 4,
            AttributeType::I32 => 1,
            AttributeType::I32I32 => 2,
            AttributeType::I32I32I32 => 3,
            AttributeType::I32I32I32I32 => 4,
            AttributeType::U32 => 1,
            AttributeType::U32U32 => 2,
            AttributeType::U32U32U32 => 3,
            AttributeType::U32U32U32U32 => 4,
            AttributeType::I64 => 1,
            AttributeType::I64I64 => 2,
            AttributeType::I64I64I64 => 3,
            AttributeType::I64I64I64I64 => 4,
            AttributeType::U64 => 1,
            AttributeType::U64U64 => 2,
            AttributeType::U64U64U64 => 3,
            AttributeType::U64U64U64U64 => 4,
            AttributeType::F16 => 1,
            AttributeType::F16F16 => 2,
            AttributeType::F16F16F16 => 3,
            AttributeType::F16F16F16F16 => 4,
            AttributeType::F16x2x2 => 4,
            AttributeType::F16x2x3 => 6,
            AttributeType::F16x2x4 => 8,
            AttributeType::F16x3x2 => 6,
            AttributeType::F16x3x3 => 9,
            AttributeType::F16x3x4 => 12,
            AttributeType::F16x4x2 => 8,
            AttributeType::F16x4x3 => 12,
            AttributeType::F16x4x4 => 16,
            AttributeType::F32 => 1,
            AttributeType::F32F32 => 2,
            AttributeType::F32F32F32 => 3,
            AttributeType::F32F32F32F32 => 4,
            AttributeType::F32x2x2 => 4,
            AttributeType::F32x2x3 => 6,
            AttributeType::F32x2x4 => 8,
            AttributeType::F32x3x2 => 6,
            AttributeType::F32x3x3 => 9,
            AttributeType::F32x3x4 => 12,
            AttributeType::F32x4x2 => 8,
            AttributeType::F32x4x3 => 12,
            AttributeType::F32x4x4 => 16,
            AttributeType::F64 => 1,
            AttributeType::F64F64 => 2,
            AttributeType::F64F64F64 => 3,
            AttributeType::F64F64F64F64 => 4,
            AttributeType::F64x2x2 => 4,
            AttributeType::F64x2x3 => 6,
            AttributeType::F64x2x4 => 8,
            AttributeType::F64x3x2 => 6,
            AttributeType::F64x3x3 => 9,
            AttributeType::F64x3x4 => 12,
            AttributeType::F64x4x2 => 8,
            AttributeType::F64x4x3 => 12,
            AttributeType::F64x4x4 => 16,
            AttributeType::I2I10I10I10Reversed => 4,
            AttributeType::U2U10U10U10Reversed => 4,
            AttributeType::I10I10I10I2 => 4,
            AttributeType::U10U10U10U2 => 4,
            AttributeType::F10F11F11UnsignedIntReversed => 3,
            AttributeType::FixedFloatI16U16 => 1,
        }
    }
}

#[derive(Debug)]
pub struct VertexAtrib {
    pub location: usize,
    pub ty: AttributeType,
    pub offset: usize,
}

pub type VertexFormat = &'static [VertexAtrib];

unsafe impl Attribute for i8 {
    const TYPE: AttributeType = AttributeType::I8;
}

unsafe impl Attribute for (i8, i8) {
    const TYPE: AttributeType = AttributeType::I8I8;
}

unsafe impl Attribute for [i8; 2] {
    const TYPE: AttributeType = AttributeType::I8I8;
}

unsafe impl Attribute for (i8, i8, i8) {
    const TYPE: AttributeType = AttributeType::I8I8I8;
}

unsafe impl Attribute for [i8; 3] {
    const TYPE: AttributeType = AttributeType::I8I8I8;
}

unsafe impl Attribute for (i8, i8, i8, i8) {
    const TYPE: AttributeType = AttributeType::I8I8I8I8;
}

unsafe impl Attribute for [i8; 4] {
    const TYPE: AttributeType = AttributeType::I8I8I8I8;
}

unsafe impl Attribute for u8 {
    const TYPE: AttributeType = AttributeType::U8;
}

unsafe impl Attribute for (u8, u8) {
    const TYPE: AttributeType = AttributeType::U8U8;
}

unsafe impl Attribute for [u8; 2] {
    const TYPE: AttributeType = AttributeType::U8U8;
}

unsafe impl Attribute for (u8, u8, u8) {
    const TYPE: AttributeType = AttributeType::U8U8U8;
}

unsafe impl Attribute for [u8; 3] {
    const TYPE: AttributeType = AttributeType::U8U8U8;
}

unsafe impl Attribute for (u8, u8, u8, u8) {
    const TYPE: AttributeType = AttributeType::U8U8U8U8;
}

unsafe impl Attribute for [u8; 4] {
    const TYPE: AttributeType = AttributeType::U8U8U8U8;
}

unsafe impl Attribute for i16 {
    const TYPE: AttributeType = AttributeType::I16;
}

unsafe impl Attribute for (i16, i16) {
    const TYPE: AttributeType = AttributeType::I16I16;
}

unsafe impl Attribute for [i16; 2] {
    const TYPE: AttributeType = AttributeType::I16I16;
}

unsafe impl Attribute for (i16, i16, i16) {
    const TYPE: AttributeType = AttributeType::I16I16I16;
}

unsafe impl Attribute for [i16; 3] {
    const TYPE: AttributeType = AttributeType::I16I16I16;
}

unsafe impl Attribute for (i16, i16, i16, i16) {
    const TYPE: AttributeType = AttributeType::I16I16I16I16;
}

unsafe impl Attribute for [i16; 4] {
    const TYPE: AttributeType = AttributeType::I16I16I16I16;
}

unsafe impl Attribute for u16 {
    const TYPE: AttributeType = AttributeType::U16;
}

unsafe impl Attribute for (u16, u16) {
    const TYPE: AttributeType = AttributeType::U16U16;
}

unsafe impl Attribute for [u16; 2] {
    const TYPE: AttributeType = AttributeType::U16U16;
}

unsafe impl Attribute for (u16, u16, u16) {
    const TYPE: AttributeType = AttributeType::U16U16U16;
}

unsafe impl Attribute for [u16; 3] {
    const TYPE: AttributeType = AttributeType::U16U16U16;
}

unsafe impl Attribute for (u16, u16, u16, u16) {
    const TYPE: AttributeType = AttributeType::U16U16U16U16;
}

unsafe impl Attribute for [u16; 4] {
    const TYPE: AttributeType = AttributeType::U16U16U16U16;
}

unsafe impl Attribute for i32 {
    const TYPE: AttributeType = AttributeType::I32;
}

unsafe impl Attribute for (i32, i32) {
    const TYPE: AttributeType = AttributeType::I32I32;
}

unsafe impl Attribute for [i32; 2] {
    const TYPE: AttributeType = AttributeType::I32I32;
}

unsafe impl Attribute for (i32, i32, i32) {
    const TYPE: AttributeType = AttributeType::I32I32I32;
}

unsafe impl Attribute for [i32; 3] {
    const TYPE: AttributeType = AttributeType::I32I32I32;
}

unsafe impl Attribute for (i32, i32, i32, i32) {
    const TYPE: AttributeType = AttributeType::I32I32I32I32;
}

unsafe impl Attribute for [i32; 4] {
    const TYPE: AttributeType = AttributeType::I32I32I32I32;
}

unsafe impl Attribute for u32 {
    const TYPE: AttributeType = AttributeType::U32;
}

unsafe impl Attribute for (u32, u32) {
    const TYPE: AttributeType = AttributeType::U32U32;
}

unsafe impl Attribute for [u32; 2] {
    const TYPE: AttributeType = AttributeType::U32U32;
}

unsafe impl Attribute for (u32, u32, u32) {
    const TYPE: AttributeType = AttributeType::U32U32U32;
}

unsafe impl Attribute for [u32; 3] {
    const TYPE: AttributeType = AttributeType::U32U32U32;
}

unsafe impl Attribute for (u32, u32, u32, u32) {
    const TYPE: AttributeType = AttributeType::U32U32U32U32;
}

unsafe impl Attribute for [u32; 4] {
    const TYPE: AttributeType = AttributeType::U32U32U32U32;
}

unsafe impl Attribute for i64 {
    const TYPE: AttributeType = AttributeType::I64;
}

unsafe impl Attribute for (i64, i64) {
    const TYPE: AttributeType = AttributeType::I64I64;
}

unsafe impl Attribute for [i64; 2] {
    const TYPE: AttributeType = AttributeType::I64I64;
}

unsafe impl Attribute for (i64, i64, i64) {
    const TYPE: AttributeType = AttributeType::I64I64I64;
}

unsafe impl Attribute for [i64; 3] {
    const TYPE: AttributeType = AttributeType::I64I64I64;
}

unsafe impl Attribute for (i64, i64, i64, i64) {
    const TYPE: AttributeType = AttributeType::I64I64I64I64;
}

unsafe impl Attribute for [i64; 4] {
    const TYPE: AttributeType = AttributeType::I64I64I64I64;
}

unsafe impl Attribute for u64 {
    const TYPE: AttributeType = AttributeType::U64;
}

unsafe impl Attribute for (u64, u64) {
    const TYPE: AttributeType = AttributeType::U64U64;
}

unsafe impl Attribute for [u64; 2] {
    const TYPE: AttributeType = AttributeType::U64U64;
}

unsafe impl Attribute for (u64, u64, u64) {
    const TYPE: AttributeType = AttributeType::U64U64U64;
}

unsafe impl Attribute for [u64; 3] {
    const TYPE: AttributeType = AttributeType::U64U64U64;
}

unsafe impl Attribute for (u64, u64, u64, u64) {
    const TYPE: AttributeType = AttributeType::U64U64U64U64;
}

unsafe impl Attribute for [u64; 4] {
    const TYPE: AttributeType = AttributeType::U64U64U64U64;
}

unsafe impl Attribute for f32 {
    const TYPE: AttributeType = AttributeType::F32;
}

unsafe impl Attribute for (f32, f32) {
    const TYPE: AttributeType = AttributeType::F32F32;
}

unsafe impl Attribute for [f32; 2] {
    const TYPE: AttributeType = AttributeType::F32F32;
}

unsafe impl Attribute for (f32, f32, f32) {
    const TYPE: AttributeType = AttributeType::F32F32F32;
}

unsafe impl Attribute for [f32; 3] {
    const TYPE: AttributeType = AttributeType::F32F32F32;
}

unsafe impl Attribute for (f32, f32, f32, f32) {
    const TYPE: AttributeType = AttributeType::F32F32F32F32;
}

unsafe impl Attribute for [f32; 4] {
    const TYPE: AttributeType = AttributeType::F32F32F32F32;
}

unsafe impl Attribute for [[f32; 2]; 2] {
    const TYPE: AttributeType = AttributeType::F32x2x2;
}

unsafe impl Attribute for [[f32; 3]; 3] {
    const TYPE: AttributeType = AttributeType::F32x3x3;
}

unsafe impl Attribute for [[f32; 4]; 4] {
    const TYPE: AttributeType = AttributeType::F32x4x4;
}

unsafe impl Attribute for f64 {
    const TYPE: AttributeType = AttributeType::F64;
}

unsafe impl Attribute for (f64, f64) {
    const TYPE: AttributeType = AttributeType::F64F64;
}

unsafe impl Attribute for [f64; 2] {
    const TYPE: AttributeType = AttributeType::F64F64;
}

unsafe impl Attribute for (f64, f64, f64) {
    const TYPE: AttributeType = AttributeType::F64F64F64;
}

unsafe impl Attribute for [f64; 3] {
    const TYPE: AttributeType = AttributeType::F64F64F64;
}

unsafe impl Attribute for (f64, f64, f64, f64) {
    const TYPE: AttributeType = AttributeType::F64F64F64F64;
}

unsafe impl Attribute for [f64; 4] {
    const TYPE: AttributeType = AttributeType::F64F64F64F64;
}

unsafe impl Attribute for [[f64; 2]; 2] {
    const TYPE: AttributeType = AttributeType::F64x2x2;
}

unsafe impl Attribute for [[f64; 3]; 3] {
    const TYPE: AttributeType = AttributeType::F64x3x3;
}

unsafe impl Attribute for [[f64; 4]; 4] {
    const TYPE: AttributeType = AttributeType::F64x4x4;
}
