use fastnbt::error::Result;
use fastnbt::from_bytes;
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Read};
use std::path::Path;

#[derive(Deserialize, Debug)]
struct MapDat<'a> {
    #[serde(borrow)]
    data: MapData<'a>,
}

#[derive(Deserialize, Debug)]
struct MapData<'a> {
    colors: &'a [u8],
}

// https://minecraft.wiki/w/Map_item_format#Base_colors
#[rustfmt::skip]
const MAP_BASE_COLORS: [[u8; 4]; 64] = [
    [0, 0, 0, 0],         // 0: air, void air, cave air, barrier, cake (including cake with candles), powered rail, detector rail, torches, redstone wire, ladder, rail, lever, buttons, repeater, tripwire hook, tripwire, flower pot (including potted plants), head, comparator, activator rail, end rod, glass, glass pane, nether portal, stained glass pane (all colors), structure void, iron bars, chain, light block, pink petals,‌[BE only] pointed dripstone,‌[BE only] redstone lamp‌[BE only]
    [127, 178, 56, 255],  // 1: grass block, slime block
    [247, 233, 163, 255], // 2: sand, suspicious sand, birch (planks, log (vertical), stripped log, wood, stripped wood, sign, pressure plate, trapdoor, stairs, slab, fence gate, fence, door), sandstone (all variants, all slabs, all stairs, all walls), glowstone, end stone, end stone bricks (slab, stairs, wall), bone block, turtle egg, scaffolding, candle, ochre froglight, frogspawn
    [199, 199, 199, 255], // 3: cobweb, mushroom stem, bed (head), white candle
    [255, 0, 0, 255],     // 4: lava, TNT, fire, redstone block
    [160, 160, 255, 255], // 5: ice, frosted ice, packed ice, blue ice
    [167, 167, 167, 255], // 6: block of iron, iron door, brewing stand, heavy weighted pressure plate, iron trapdoor, lantern, anvil (all damage levels), grindstone, lodestone,‌[JE only] heavy core, pale oak leaves, pale oak sapling, copper trapdoor‌[BE only] (including waxed and exposed/weathered variations)
    [0, 124, 0, 255],     // 7: sapling, flowers, wheat, sugar cane, pumpkin stem, melon stem, lily pad, cocoa, carrots, potatoes, beetroots, sweet berry bush, grass,‌[JE only] tall grass, fern,‌[JE only] large fern, vines,‌[JE only] leaves‌[JE only] (except pale oak leaves, cherry leaves), cactus, bamboo, cave vines, spore blossom, (flowering) azalea, dripleaf (big and small), pink petals,‌[JE only] wildflowers, bush, torchflower seeds, pitcher pod, hanging roots‌[BE only]
    [255, 255, 255, 255], // 8: snow, snow block, white (bed (foot), wool, stained glass, carpet, shulker box, glazed terracotta, concrete, concrete powder), powder snow, lodestone‌[BE only]
    [164, 168, 184, 255], // 9: clay, heavy core
    [151, 109, 77, 255],  // 10: dirt, coarse dirt, farmland, dirt path, granite (slab, stairs, wall), polished granite (slab, stairs), jungle (planks, log (vertical), stripped log, wood, stripped wood, sign, pressure plate, trapdoor, stairs, slab, fence gate, fence, door), jukebox, brown mushroom block, rooted dirt, hanging roots,‌[JE only] packed mud
    [112, 112, 112, 255], // 11: stone (slab, stairs), andesite (slab, stairs, wall), polished andesite (slab, stairs), cobblestone (slab, stairs, wall), bedrock, gold ore, iron ore, coal ore, lapis lazuli ore, dispenser, mossy cobblestone (slab, stairs, wall), monster spawner, diamond ore, furnace, stone pressure plate, redstone ore, stone bricks (all variants, all slabs, all stairs, all walls), emerald ore, ender chest, dropper, smooth stone (slab), observer, smoker, blast furnace, stonecutter, sticky piston, piston, piston head, gravel, suspicious gravel, acacia log (side), cauldron (including cauldrons with water, lava, or powdered snow), hopper, copper ore, crafter, vault, trial spawner, pale oak wood, pale oak log (side), infested block‌[BE only] (not including infested deepslate), heavy core
    [64, 64, 255, 255],   // 12: kelp, seagrass, water, bubble column, waterlogged leaves
    [143, 119, 72, 255],  // 13: oak (planks, log (vertical), stripped log, wood, stripped wood, sign, door, pressure plate, fence, trapdoor, fence gate, slab, stairs), note block, bookshelf, chiseled bookshelf, chest, crafting table, trapped chest, daylight detector, loom, barrel, cartography table, fletching table, lectern, smithing table, composter, bamboo sapling, dead bush, petrified oak slab, beehive, banners (all colors, when not as markers)
    [255, 252, 245, 255], // 14: diorite (stairs, slab, wall), polished diorite (stairs, slab), birch log (side), quartz block (all variants, all slabs, all stairs), sea lantern, target, pale oak (planks, log (vertical), stripped log, stripped wood, sign, pressure plate, trapdoor, stairs, slab, fence gate, fence, door)
    [216, 127, 51, 255],  // 15: acacia (planks, log (vertical), stripped log, stripped wood,‌[JE only] sign, trapdoor, slab, stairs, pressure plate, fence gate, fence, door), red sand, orange (wool, carpet, shulker box, bed (foot), stained glass, glazed terracotta, concrete, concrete powder, candle), pumpkin, carved pumpkin, jack o'lantern, terracotta, red sandstone (all variants, all stairs, all slabs, all walls), honey block, honeycomb block, block of copper (including all cut, waxed, stair, trapdoor,‌[JE only] and slab variants), lightning rod, block of raw copper, creaking heart (all orientations, active & inactive), open eyeblossom
    [178, 76, 216, 255],  // 16: magenta (wool, carpet, shulker box, bed (foot), stained glass, glazed terracotta, concrete, concrete powder, candle), purpur (all variants, slab, stairs)
    [102, 153, 216, 255], // 17: light blue (wool, carpet, shulker box, bed (foot), stained glass, glazed terracotta, concrete, concrete powder, candle), soul fire
    [229, 229, 51, 255],  // 18: yellow (wool, carpet, shulker box, bed (foot), stained glass, glazed terracotta, concrete, concrete powder, candle), bamboo (planks, stripped, sign, door, pressure plate, fence, trapdoor, fence gate, slab, stairs), bamboo mosaic (slab, stairs), sponge, wet sponge, hay bale, horn coral (coral block, coral,‌[JE only] coral fan‌[JE only]), bee nest, short dry grass, tall dry grass
    [127, 204, 25, 255],  // 19: lime (wool, carpet, shulker box, bed (foot), stained glass, glazed terracotta, concrete, concrete powder, candle), melon
    [242, 127, 165, 255], // 20: pink (wool, carpet, shulker box, bed (foot), stained glass, glazed terracotta, concrete, concrete powder, candle), brain coral (coral block, coral,‌[JE only] coral fan‌[JE only]), pearlescent froglight, cherry leaves, cherry sapling, cactus flower
    [76, 76, 76, 255],    // 21: gray (wool, carpet, shulker box, bed (foot), stained glass, glazed terracotta, concrete, concrete powder, candle), acacia wood, stripped wood,‌[BE only] dead coral (coral block, coral, coral fan), tinted glass
    [153, 153, 153, 255], // 22: light gray (wool, carpet, shulker box, bed (foot), stained glass, glazed terracotta, concrete, concrete powder, candle), structure block, jigsaw block, pale moss carpet, pale moss block, test block‌[JE only]
    [76, 127, 153, 255],  // 23: cyan (wool, carpet, shulker box, bed (foot), stained glass, glazed terracotta, concrete, concrete powder, candle), prismarine (slab, stairs, wall), warped (roots, fungus), twisting vines, nether sprouts, sculk sensor,‌[JE only] warped (pressure plate,‌[BE only] sign,‌[BE only] slab,‌[BE only] trapdoor,‌[BE only] fence‌[BE only])
    [127, 63, 178, 255],  // 24: shulker box, purple (wool, carpet, shulker box,‌[BE only] bed (foot), stained glass, glazed terracotta, concrete, concrete powder, candle), mycelium, chorus plant, chorus flower, repeating command block, bubble coral (coral block, coral,‌[JE only] coral fan‌[JE only]), amethyst block, budding amethyst, amethyst cluster, amethyst bud (all sizes)
    [51, 76, 178, 255],   // 25: blue (wool, carpet, shulker box, bed (foot), stained glass, glazed terracotta, concrete, concrete powder, candle), tube coral (coral block, coral,‌[JE only] coral fan‌[JE only])
    [102, 76, 51, 255],   // 26: brown (wool, carpet, shulker box, bed (foot), stained glass, glazed terracotta, concrete, concrete powder, candle), dark oak (planks, log, stripped log, wood, stripped wood, sign, pressure plate, trapdoor, stairs, slab, fence gate, fence, door), soul sand, command block, brown mushroom, soul soil, leaf litter
    [102, 127, 51, 255],  // 27: green (wool, carpet, shulker box, bed (foot), stained glass, glazed terracotta, concrete, concrete powder, candle), end portal frame, chain command block, sea pickle, moss carpet, moss block, dried kelp block
    [153, 51, 51, 255],   // 28: red (wool, carpet, shulker box, bed (foot), stained glass, glazed terracotta, concrete, concrete powder, candle), bricks (slab, stairs, wall), red mushroom block, nether wart, enchanting table, nether wart block, fire coral (coral block, coral,‌[JE only] coral fan‌[JE only]), red mushroom, shroomlight, mangrove (planks, log (vertical), stripped log, wood, stripped wood, sign, door, pressure plate, fence, trapdoor, fence gate, slab, stairs), sniffer egg
    [25, 25, 25, 255],    // 29: black (wool, carpet, shulker box, bed (foot), stained glass, glazed terracotta, concrete, concrete powder, candle), obsidian, end portal, dragon egg, block of coal, end gateway, basalt, polished basalt, smooth basalt, block of netherite, ancient debris, crying obsidian, respawn anchor, blackstone (all variants, all stairs, all slabs, all walls), gilded blackstone, sculk, sculk vein, sculk catalyst, sculk shrieker, sculk sensor‌[BE only]
    [250, 238, 77, 255],  // 30: block of gold, light weighted pressure plate, bell, block of raw gold
    [92, 219, 213, 255],  // 31: block of diamond, beacon, prismarine bricks (slab, stairs), dark prismarine (slab, stairs), conduit
    [74, 128, 255, 255],  // 32: block of lapis lazuli
    [0, 217, 58, 255],    // 33: block of emerald
    [129, 86, 49, 255],   // 34: podzol, spruce (planks, log (vertical), stripped log, wood, stripped wood, sign, pressure plate, trapdoor, stairs, slab, fence gate, fence, door), oak log (side), jungle log (side), campfire, soul campfire, mangrove log (side), mangrove roots, muddy mangrove roots
    [112, 2, 0, 255],     // 35: netherrack, nether bricks (fence, slab, stairs, wall, chiseled, cracked), nether gold ore, nether quartz ore, magma block, red nether bricks (slab, stairs, wall), crimson (roots, fungus), weeping vines, crimson (pressure plate,‌[BE only] sign,‌[BE only] slab,‌[BE only] trapdoor,‌[BE only] fence‌[BE only])
    [209, 177, 161, 255], // 36: white terracotta, calcite, cherry (planks, log (vertical), stripped log (vertical), sign, pressure plate, trapdoor, stairs, slab, fence gate, fence, door)
    [159, 82, 36, 255],   // 37: orange terracotta, redstone lamp,‌[JE only] resin bricks (fence, slab, stairs, wall, chiseled), block of resin
    [149, 87, 108, 255],  // 38: magenta terracotta
    [112, 108, 138, 255], // 39: light blue terracotta
    [186, 133, 36, 255],  // 40: yellow terracotta
    [103, 117, 53, 255],  // 41: lime terracotta
    [160, 77, 78, 255],   // 42: pink terracotta, cherry (stripped log (side), stripped wood)
    [57, 41, 35, 255],    // 43: gray terracotta, cherry log (side), cherry wood, tuff (all variants, all slabs, all stairs, all walls)
    [135, 107, 98, 255],  // 44: light gray terracotta, exposed copper (including all cut, waxed, stair, trapdoor‌[JE only] and slab variants), mud bricks (slab, stairs, wall)
    [87, 92, 92, 255],    // 45: cyan terracotta, mud
    [122, 73, 88, 255],   // 46: purple (terracotta, shulker box‌[JE only])
    [76, 62, 92, 255],    // 47: blue terracotta
    [76, 50, 35, 255],    // 48: brown terracotta, pointed dripstone,‌[JE only] dripstone block
    [76, 82, 42, 255],    // 49: green terracotta, closed eyeblossom
    [142, 60, 46, 255],   // 50: red terracotta, decorated pot
    [37, 22, 16, 255],    // 51: black terracotta
    [189, 48, 49, 255],   // 52: crimson nylium
    [148, 63, 97, 255],   // 53: crimson (door, stairs, stem, stripped stem, fence gate, planks, pressure plate,‌[JE only] sign,‌[JE only] slab,‌[JE only] trapdoor,‌[JE only] fence‌[JE only])
    [92, 25, 29, 255],    // 54: crimson (hyphae, stripped hyphae)
    [22, 126, 134, 255],  // 55: warped nylium, oxidized copper (including all cut, waxed, stair, trapdoor,‌[JE only] and slab variants)
    [58, 142, 140, 255],  // 56: warped (door, stairs, stem, stripped stem, fence gate, planks, pressure plate,‌[JE only] sign,‌[JE only] slab,‌[JE only] trapdoor,‌[JE only] fence‌[JE only]), weathered copper (including all cut, waxed, stair, trapdoor,‌[JE only] and slab variants)
    [86, 44, 62, 255],    // 57: warped (hyphae, stripped hyphae)
    [20, 180, 133, 255],  // 58: warped wart block
    [100, 100, 100, 255], // 59: deepslate (gold ore, iron ore, coal ore, lapis ore, diamond ore, redstone ore, emerald ore, copper ore), deepslate (all variants, all stairs, all slabs, and all walls), infested deepslate, reinforced deepslate
    [216, 175, 147, 255], // 60: block of raw iron
    [127, 167, 150, 255], // 61: glow lichen, verdant froglight
    [0, 0, 0, 0],         // 62: unused
    [0, 0, 0, 0],         // 63: unused
];

// https://minecraft.wiki/w/Map_item_format#Map_colors
const MAP_COLOR_MULTIPLIERS: [u8; 4] = [180, 220, 255, 135];

fn main() {
    let args: Vec<String> = env::args().collect();

    for arg in args {
        let map_dat_path = Path::new(&arg);
        let mut map_dat_raw = vec![];
        let map_dat: MapDat;
        let colors: &[u8];
        let map_length: u32;
        let map_width: u32;

        // Load that map_nnn.dat!
        {
            let Some(extension) = map_dat_path.extension() else {
                continue;
            };

            if extension != "dat" {
                continue;
            };

            let Ok(file) = std::fs::File::open(map_dat_path) else {
                eprintln!("Failed to open: {}", map_dat_path.display());
                continue;
            };

            let mut decoder = GzDecoder::new(file);
            let Ok(_) = decoder.read_to_end(&mut map_dat_raw) else {
                eprintln!("Failed to decompress: {}", map_dat_path.display());
                continue;
            };

            let map_dat_result: Result<MapDat> = from_bytes(map_dat_raw.as_slice());
            let Ok(map_dat_temp) = map_dat_result else {
                eprintln!("Failed to decode as map data: {}", map_dat_path.display());
                continue;
            };

            map_dat = map_dat_temp;
            colors = map_dat.data.colors;

            map_width = (colors.len() as f64).sqrt() as u32;
            map_length = map_width * map_width;

            if colors.len() != (map_length as usize) {
                eprintln!("Not a square map?: {}", map_dat_path.display());
                continue;
            }
        }

        // Now we can create that PNG~ :3
        {
            let map_png_path = map_dat_path.with_extension("png");
            let Ok(file) = File::create(&map_png_path) else {
                eprintln!("Failed to create file: {}", map_png_path.display());
                continue;
            };

            let ref mut w = BufWriter::new(file);
            let mut encoder = png::Encoder::new(w, map_width, map_width);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455));
            encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
            let source_chromaticities = png::SourceChromaticities::new(
                (0.31270, 0.32900),
                (0.64000, 0.33000),
                (0.30000, 0.60000),
                (0.15000, 0.06000),
            );
            encoder.set_source_chromaticities(source_chromaticities);
            let Ok(mut writer) = encoder.write_header() else {
                eprintln!("Failed to write png header: {}", map_png_path.display());
                continue;
            };

            let mut data = vec![0; (map_length * 4) as usize];

            for i in 0..map_length {
                let i = i as usize;
                // https://minecraft.wiki/w/Map_item_format#Map_colors
                let map_color_id = colors[i] as usize;
                let map_base_color_id = map_color_id / 4;
                let map_color_multiplier_id = map_color_id % 4;
                data[i * 4 + 0] = ((MAP_BASE_COLORS[map_base_color_id][0] as u16)
                    * (MAP_COLOR_MULTIPLIERS[map_color_multiplier_id] as u16)
                    / 255) as u8;
                data[i * 4 + 1] = ((MAP_BASE_COLORS[map_base_color_id][1] as u16)
                    * (MAP_COLOR_MULTIPLIERS[map_color_multiplier_id] as u16)
                    / 255) as u8;
                data[i * 4 + 2] = ((MAP_BASE_COLORS[map_base_color_id][2] as u16)
                    * (MAP_COLOR_MULTIPLIERS[map_color_multiplier_id] as u16)
                    / 255) as u8;
                data[i * 4 + 3] = MAP_BASE_COLORS[map_base_color_id][3];
            }

            let Ok(()) = writer.write_image_data(&data) else {
                eprintln!("Failed to write image data: {}", map_png_path.display());
                continue;
            };
        }
    }
}
