# General
## Direct State Access
- Faster, no binds
## Buffers
### Persistent Mapping
- Fastest way to write data to GPU buffer on NVIDIA GPU *(GDC, 2014)*

### N-Buffering
- No stutter when modifying buffer data

# Mesh approach
## Render one cube per draw call
- Slow, lots of draw calls.

## Render all cubes per draw call
Significantly faster

## Use chunks to pack data into a 32bit int
- Less data sent to GPU
	- Can pack local position, normal and block type into 32bits
	- Send over chunk position in a uniform
- More draw calls
	- Need a draw call for every chunk
- Use 32$^3$ block chunks so they fit neatly into a 32 bit int (for later)

## Cull inner faces
- Need to process data
	- Takes time, worth it? Yes
- Significantly less triangles
	- By orders of magnitude
	- Can use a tri strip with a plane instance

- Create depth mask per axis (3) of `[[i32; 32]; 32]` with depth along the int
- Bit shift in a 1 for each solid voxel `axes[axis][z][x] &= 1 << y` with `x` and `z` being coordinates on a plane, and y being the depth.
- Cull depths by checking bits patterns of 01 for forward faces or 10 for backward faces
	- Can do this with a bitwise and against a negated shifted mask 

	```rust
	// Convert the enum into an index
	let axis_index = usize::from(axis);

	// Get the depth mask for the axis at coordinates (x, z) 
	let depth_mask: u32 = axis_depths[usize::from(axis)][z][x];
	
	// Get a mask for occurances of 01
	let zero_one_mask = col & !(col >> 1);

	// Get a mask for occurances of 10
	let one_zero_mask = col & !(col << 1);
	```
	
	- Create a depth mask per face of `[[i32; 32]; 32]` with depth along the int. Can convert the axis index to a face index.

```rust
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