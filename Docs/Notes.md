# Terminology
**Voxel**: Data associated with a point in 3D space e.g. a colour at point [1, 2, 3]. Short for *Volumetric Pixel*. Usually represented as a cube stretching from point to point + 1.
> A **voxel** is a three-dimensional counterpart to a [pixel](https://en.wikipedia.org/wiki/Pixel "Pixel"). It represents a value on a [regular grid](https://en.wikipedia.org/wiki/Regular_grid "Regular grid") in a [three-dimensional space](https://en.wikipedia.org/wiki/Three-dimensional_space "Three-dimensional space"). - [Wikipedia](https://en.wikipedia.org/wiki/Voxel)

**LOD**: Level of detail. In relation to voxels it refers to combining a 2$^3$ (or larger) area into one voxel instead of multiple. Used to reduce quality of scenery in the distance where you won't notice.
**SSBO**: Shader Storage Buffer Object. A section of data stored on the GPU.
**N-Buffering**: Using multiple buffers for the same data. If you write to a buffer while it's in use the program will wait for the write to finish - causing a stutter. So use multiple so you can use one while writing to the other(s), then swap which you're using when the write finishes.
**Frustum Culling**: A frustum is a cone / pyramid with the tip chopped off - this is a mathematical representation of what a camera can see. Frustum culling usually refers to "culling" (not drawing) objects that are outside of the cameras view (frustum).

# General
## Direct State Access
- Faster, don't need to bind a buffer to write to it
## Buffers
### Persistent Mapping
- Fastest way to write data to GPU buffer on NVIDIA GPU *(AZDO, GDC, 2014)*

### N-Buffering
- #h/green **Pros**
	- No stutter when modifying buffer data
- #h/red **Cons**
	- Need to deal with multiple buffers
	- Uses more memory
	- Will use stale data for a few frames
- #h/cyan **Notes**
	- Can be used on a mesh level (n-meshing), but I used it for instances and SSBO's only

# Mesh approach
- #h/green **Pros**
	- Easy to get started
	- Fits into an existing engine very well
	- Can optimize very well for static environments #h/pink(*Can't place / break blocks*)

- #h/red **Cons**
	- Hard to implement LOD

- #h/cyan **Notes**
	- Performance can vary wildly with different environments, e.g. greedy meshing works best with lots of large flat planes, and will suffer greatly with something like a checkerboard.
## Render one cube per draw call
- #h/green **Pros**
	- Very simple

- #h/red **Cons**
	- Slow, lots of draw calls. #h/pink(*Like asking "Can I have an apple" 10 times*)

- #h/cyan **Notes**
	- Store position, normal and colour in vertex

## Render all cubes per draw call
- #h/green **Pros**
	- Very simple, uses instancing #h/pink(*Like asking "Can I have 10 apples" once*)
	- Straight upgrade to previous

- #h/red **Cons**
	- Hard to do frustum culling
	- Hard to implement a render distance

- #h/cyan **Notes**
	- Store vertex position and normals in vertex
	- Store voxel position and colour in instance

## Use chunks to pack data into a 32bit int
- #h/green **Pros**
	- Less data sent to GPU
		- Can pack local position, normal and block type into 32bits
		- Send over chunk position in a uniform
	- Easy frustum culling, can check if any of the chunk is in the frustum instead of individual blocks.
	- Easy render distance
- #h/red **Cons**
	- More draw calls
		- Need a draw call for every chunk
- #h/cyan **Notes**
	- Use 32$^3$ block chunks so they fit neatly into a 32 bit int (for later)
		- Vertex position in vertex
		- Chunk space voxel position, normals and colour in instance
		- Chunk potion in uniforms

## Cull inner faces
- #h/green **Pros**
	- Significantly less triangles
		- By orders of magnitude
- #h/red **Cons**
	- Need to process data
		- Takes time, worth it? Yes

- Create depth mask per axis (3) of `[[i32; 32]; 32]` with depth along the int
- Bit shift in a 1 for each solid voxel `axes[axis][z][x] &= 1 << y` with `x` and `z` being coordinates on a plane, and y being the depth.
- Cull depths by checking bits patterns of 01 for forward faces or 10 for backward faces
	- Can do this with a bitwise and against a negated shifted mask 
```
         Shift    Negate
01110    01110    01110
01110 &  11100 &  00011 &
01110    01100    00010
```

```rust
	for axis in Dir::all() {
		for z in 0..CHUNK_SIZE {
			for x in 0..CHUNK_SIZE {
				// Convert the enum into an index
				let axis_index = usize::from(axis);
			
				// Get the depth mask for the axis at coordinates (x, z) 
				let depth_mask: u32 = axis_depths[usize::from(axis)][z][x];
				
				// Get a mask for occurances of 01
				let zero_one_mask = col & !(col >> 1);
			
				// Get a mask for occurances of 10
				let one_zero_mask = col & !(col << 1);

				// Add mask to culled faces
				// ...
			}
		}
	}
```

- Create a depth mask per face of `[[i32; 32]; 32]` with depth along the int. Can convert the axis index to a face index.

```rust
// Add mask to culled faces
culled_face_depths[2 * axis_index + 0][z][x] = one_zero_mask;
culled_face_depths[2 * axis_index + 1][z][x] = zero_one_mask;
```

- Need to transpose the culled face depths to have either the `x` or `z` axis along the int. I chose to use the `z` axis
```rust
for face in Dir::all() {
	let face_index = usize::from(face);
	
	for z in 0..CHUNK_SIZE {
		for x in 0..CHUNK_SIZE {
			let mut col = culled_face_depths[face][z][x];

			while col != 0 {
				let y = col.trailing_zeros() as usize;

				// Clear least significant set bit so we can access the next set bit
				col &= col - 1;

				culled_faces[face_index][y][x] |= 1 << z;
			}
		}
	}
}
```
## Combine faces
- Even more processing
	- Worth it? Maybe
	- Need to pack width and height of face into instance data
	- Can only combine faces with same texture and AO
- Even less triangles, not as much of a jump

## Frustum Culling
- Cheap, can use hierarchies to check quicker. Eliminates a lot of triangles before even being sent to GPU.

## Draw all chunks with one draw call *(GDC, 2014)*
- Can batch with `MultiDraw*Indirect`
- Can dispatch indirect creation to a compute shader for no CPU-GPU sync