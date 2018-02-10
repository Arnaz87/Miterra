
# Todo

- Finish automatic chunk generation
- Fix bad chunk border normals
- Improve voxel smoothing (probably fixes previous issue)
- Fix bland textures, probably due to the srgb issue
- Voxel editing
- Arbitrary materials
- Decent terrain generation
- Grass
- Sky
- Physics
- Bumpmap based material transition

# Long term

- Model loading
- Multiplayer
- Biomes support
- Modding api
- Crafting system

# Assets

grass and soilsand textures are provided by www.textures.com

# Mapgen ideas

First a large scale 2D noise that controls general terrain behaviour, big values are mountains and small values are plain areas. A smaller 2D noise controls actual terrain height, it applies everywhere but in mountanous areas it's effects are exagerated. After generationg that, subtract caves.

A single or double voxel layer of a top material, like grass, can be added to the terrain. But in far chunks a layer that small will alias and won't be seen, due to closest neighbour voxels selected for low LOD voxels, and those voxels very likely won't be one of that layer voxels. To avoid that, any low LOD voxel that is touching air could find in it's actual contained voxels for the material that is touching air, and replace it's closest-neighbour voxel with that material. It would look by starting at the xz coordinates of the voxel and look from top to bottom until a solid voxel is found. If it's not touching air vertically, then this process can be skipped, as this is only a problem with thin layers of grass on top of the terrain.

For low LOD voxels made of manually edited voxels, it could scan all internal voxels and first, it's solid if more than half of its voxels are, and if it touches air, repeat the same process described above (look for the actual air touching voxel), or else select the majority material.

