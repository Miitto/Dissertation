<!-- markdownlint-configure-file { "MD013": false } -->

# Optimizations for a voxel engine

## Introduction

### Context

Standard rendering approaches work with pre-defined meshes, which can be statically optimized by the developers before being included in the game. They can manipulate the vertices through simple displacement, possibly guided via animations set up using rigging that was set up previously in 3D modeling software. This results in the final product being a "skin" around the objects which, while efficient, do not allow for in depth modifications at runtime. Voxels are a different approach that allows for depth to be expressed dynamically at runtime. With a standard rendering approach, if you cut a hole in the mesh there would be nothing behind, whereas with a voxel engine you would be able to reveal the terrain behind.

### Problem

Due to the nature of voxels, you need to transform the stored voxel data into something you can render to the screen. There are two main approaches to this: meshing and raycasting. Meshing involves transforming the voxel data into a triangle mesh akin to a standard rendering approach. This approach has a simple starting point but is extremely slow if not optimized. Additionally the mesh needs to be rebuilt every time the voxel data is modified which incurs further performance penalties at runtime. Raycasting is a significantly more complex process that poses its own challenges beyond the initial complexity of the raycasting itself - sending the voxel data to the GPU efficiently for instance.

### Rationale

This project aims to show different techniques to optimize a voxel engine while comparing them to highlight where each optimization excels and any pitfalls they present. I will start with a basic implementation of a meshed voxel engine that draws a single cube for every voxel, then proceed to optimize it step by step. Each optimization should not rely on decreasing visual quality so each optimization will need to be compatible with lighting techniques such as shadow mapping and ambient occlusion. These should result in the implementations to be expanded upon to allow more advanced visual effects. The optimizations will be focused on the voxel engine itself, or how traditional optimizations can be adapted or improved for use with a voxel engine.

## Key Background Sources

| Resource                                                                                                           | Description                                                                                                                              | Reason                                                                                                                                                                            |
| ------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Advanced graphics programming using OpenGL [1]                                                                     | This book covers a significant amount of advanced techniques in OpenGL and covers a range of topics from coordinate spaces to lighting.  | This book forms a basis for the core render pipeline and the generic lighting implementation used to ensure that an optimization does not prevent the use of lighting techniques. |
| Interactive indirect illumination using voxel cone tracing [2]                                                     | This paper covers using cone tracing with voxels to estimate indirect illumination and ambient occlusion.                                | This paper showcases an approach that may be used for a more in depth lighting approach to compare how optimizations affect more advanced graphical techniques.                   |
| Efficient sparse voxel octrees [3] and Efficient sparse voxel octrees–analysis, extensions, and implementation [4] | These papers cover an implementation of sparse voxel octrees which are then streamed to the GPU and used for a raycasted voxel renderer. | These two papers cover the theory that I can use to access the voxel data on the GPU efficiently, for either a raycasted rendering approach or for lighting calculations.         |

## Aims and Objectives

### Aims

My project aims to start with an unoptimized engine with each voxel being drawn as an individual cube and then proceed to incrementally optimize the engine. Each step should explain the theory behind the implementation, how it compares to its predecessor and any detriments it may introduce. The theory should not be limited to the implementation specifics, while the project will use OpenGL for rendering, the optimizations this project explores should be applicable to other rendering APIs such as Vulkan without the need for major modifications.

### Objectives

1. Research optimizations I already know about to gain a more complete understanding and identify new optimizations I am not currently aware of.
2. Implement a core render pipeline that can support all implementations using Rust and OpenGL.
3. Implement a basic voxel engine that renders a single cube for each voxel.
4. Implement each optimization as a standalone instance using the core render pipeline setup earlier.
5. Compare the performance of each optimization using pre made test scenes for a fair comparison.

## References

[1] McReynolds, T. and Blythe, D., 2005. _Advanced graphics programming using OpenGL_. Elsevier.
[2] Crassin, C., Neyret, F., Sainz, M., Green, S. and Eisemann, E., 2011, September. Interactive indirect illumination using voxel cone tracing. In Computer Graphics Forum (Vol. 30, No. 7, pp. 1921-1930). Oxford, UK: Blackwell Publishing Ltd.
[3] Laine, S. and Karras, T., 2010, February. Efficient sparse voxel octrees. In _Proceedings of the 2010 ACM SIGGRAPH symposium on Interactive 3D Graphics and Games_ (pp. 55-63).
[4] Laine, S. and Karras, T., 2010. Efficient sparse voxel octrees–analysis, extensions, and implementation. _NVIDIA Corporation_, _2_(6).
