<!-- markdownlint-configure-file { "MD013": false } -->

# Optimizations for a voxel engine

## Introduction

### Context
Standard rendering approaches work with pre-defined meshes, which can be statically optimized by the developers before being included in the game. These meshes only represent the surface of the object.
Voxels are a different approach that allows for depth to be expressed dynamically at runtime. With a standard rendering approach, if you cut a hole in the mesh there would be nothing behind, whereas with a voxel engine you would be able to reveal the terrain behind.

### Problem
Due to the nature of voxels, the stored voxel data needs to be transformed into something that can be rendered to the screen.
There are two main approaches to this: meshing and raycasting. Meshing involves transforming the voxel data into a triangle mesh akin to a standard rendering approach. This approach has a simple starting point but is extremely slow if not optimized. Additionally the mesh needs to be rebuilt every time the voxel data is modified which incurs further performance penalties at runtime.
Raycasting is a significantly more complex process that poses its own challenges beyond the initial complexity of the raycasting itself for example sending the voxel data to the GPU efficiently.

### Rationale
This project aims to show different techniques to optimize a voxel engine by comparing them to highlight where each optimization excels and any pitfalls they present.
I will start with a basic implementation of a meshed voxel engine that draws a single cube for every voxel, then proceed to optimize it step by step. Each optimization should not decrease visual quality so each optimization will need to be compatible with lighting techniques such as shadow mapping and ambient occlusion.
These constraints should result in implementations which can be expanded upon to allow more advanced visual effects. The optimizations will be focused on the voxel engine itself, or on how traditional optimizations can be adapted or improved for use with a voxel engine.

## Key Background Sources

| Resource                                                                                                           | Description                                                                                                                                                                                                                           | Reason                                                                                                                                                                                                                                                                                                                                                                                     |
| ------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Advanced graphics programming using OpenGL [1]                                                                     | This book covers a wide variety of graphics concepts with sample implementations in OpenGL. The book is aimed at people who have a basic understanding of OpenGL and wish to expand their knowledge base.                             | This book forms a basis for the core render pipeline and the generic lighting implementation used to ensure that an optimization does not prevent the use of lighting techniques.                                                                                                                                                                                                          |
| Interactive indirect illumination using voxel cone tracing [2]                                                     | This paper covers using cone tracing to simulate global illumination and ambient occlusion through voxelizing the scene and then using a cone tracing algorithm on the created sparse voxel octree to calculate lighting information. | This paper shows a method for cone tracing using a sparse voxel octree. Since I am already working with voxels, the rasterization steps to build the sparse voxel octree are not applicable. However the rest of the paper can be applied to my needs. This will form a basis for advanced lighting used to ensure optimizations do not interfere with more advanced graphical techniques. |
| Efficient sparse voxel octrees [3] and Efficient sparse voxel octrees–analysis, extensions, and implementation [4] | These papers cover an implementation of sparse voxel octrees which is used for a raycasting based renderer for voxels.                                                                                                                | These two papers cover the theory that can be used for storing voxels for a raycasted rendering approach and cone traced lighting.                                                                                                                                                                                                                                                         |
| Greedy Meshing Voxels Fast [5]                                                                                     | This talk focuses on a method of creating a mesh from voxel data through binary greedy meshing.                                                                                                                                       | This talk forms the basis for an implementation of the binary greedy meshing algorithm. A subset of this is used for a binary culled mesher as a standalone optimization.                                                                                                                                                                                                                  |

## Aims and Objectives

### Aims
My project aims to start with an unoptimized voxel engine written with OpenGL in Rust. This engine will initially render each voxel individually as a cube and then I will incrementally implement optimizations step by step. Each step should explain the theory behind the implementation, how it compares to its predecessor and any detrimental effects.
The theory should not be limited to the implementation specifics as, whilst the project will use OpenGL for rendering, the optimizations this project explores should be applicable to other rendering APIs, such as Vulkan, without the need for major modifications.

### Objectives
1. Research and identify potential optimizations to implement in the project.
2. Implement a core rendering framework that can be used to implement all optimizations using Rust and OpenGL.
3. Implement a basic voxel engine that renders a single cube for each voxel as a benchmark to compare the optimizations against.
4. Create a number of test environments and scenarios that can be used to automate the benchmarking of the optimizations in the following categories: FPS with a stationary camera, FPS with a moving camera, voxel pre-processing time (for use to compare the speed of mesh regenerating when using a meshing algorithm).
5. Implement each optimization as a standalone instance using the core render pipeline setup earlier. All Optimizations should exist alongside each other and for it to be possible to select an optimization at compile time.
6. Compare the performance of each optimization using the scenarios and benchmarks set up in objective 4.

## Planning
```mermaid
gantt
	dateformat DD-MM
	todaymarker off
	
	section Core
		Core Render Framework :crit, core, 21-02, 5d
	section Meshers
		Port :port, after core, 4d
		Load test scenes :mload, after environ, 2d
	section Test
		Test environment creation :environ, after port, 3d
		Automated benchmarking setup :bench, after environ, until cone
	section Raycasting
		Research :rcres, after environ, 5d
		Implementation :rcim, after rcres, 10d
		Cone traced lighting :cone, 16-03, 10d
	section Future
		Future discoveries :fut, after cone, 07-04
		Slippage :slip, after fut, 14d
	section Write Up
		Dissertation :diss, after port, 07-05

```
I have given 5 days to implement the core render framework as this is the most important part of the implementation as everything else is based on it. I believe that 5 days is enough for this as I have already started working on it, and it is nearly at a usable state. This covers objective 2.

All of the meshing algorithms are already implemented and only need to be ported to work with the core render framework. This time accounts for the time it will take to port over the implementations to use the render framework, as well as any time taken in the future to tweak the implementations. Two days have been allocated to implement a method to load in the test scenes to be used for testing. This should not take too long as all meshers store their voxel data in a similar structure. Part of this period will cover objective 3 and the rest will contribute to objective 5.

Three days are allocated for creating test scenes. This includes smaller detailed scenes to test specifics in a contained environment as well as larger scenes that are created with basic procedural generation. Since this project does not focus on detail, these environments do not need to be overly complex.
The period for automated benchmarking spans the period of all implementations as I intend to implement these tests as I work.
These two periods will cover objective 4.

I have a section for dedicating my time to researching raycasting techniques. This is in preparation for the implementation afterwards as I feel it best to properly understand what I am about to implement. This is tentatively allocated 5 days although I will likely start on the implementation earlier.

The raycasting implementation and cone traced lighting overlap as they use many of the same techniques that can be reused.

I intend to start writing the main dissertation after I have finished the port of the meshing algorithms. This will allow me to write each section while it is still fresh.

The time from after cone tracing is finished until a month before the due date is dedicated to any future optimizations I discover from my ongoing research.

Two weeks of slippage is included and the final two weeks are solely dedicated to finishing the dissertation. This will safeguard from tasks taking longer than expected.

### Risks
There is a risk that I will not be able to implement the cone traced lighting in a reasonable amount of time. I have allocated time for a decent amount of runover but if that proves to be insufficient then I can abandon this feature without affecting any others. While not ideal as I intend to ensure the optimizations are compatible with more advanced graphical techniques, I can fall back to Phong Shading and Shadow Mapping that will be implemented with the core render framework.



## References
[1] - McReynolds, T. and Blythe, D., 2005. _Advanced graphics programming using OpenGL_. Elsevier.
[2] - Crassin, C., Neyret, F., Sainz, M., Green, S. and Eisemann, E., 2011, September. Interactive indirect illumination using voxel cone tracing. In Computer Graphics Forum (Vol. 30, No. 7, pp. 1921-1930). Oxford, UK: Blackwell Publishing Ltd.
[3] - Laine, S. and Karras, T., 2010, February. Efficient sparse voxel octrees. In _Proceedings of the 2010 ACM SIGGRAPH symposium on Interactive 3D Graphics and Games_ (pp. 55-63).
[4] - Laine, S. and Karras, T., 2010. Efficient sparse voxel octrees–analysis, extensions, and implementation. _NVIDIA Corporation_, _2_(6).
[5] - Davis Morley, 2022. *Greedy Meshing Voxels Fast*. Optimism in Design Handmade Seattle. [online] https://www.youtube.com/watch?v=4xs66m1Of4A (Accessed 19/02/2025).