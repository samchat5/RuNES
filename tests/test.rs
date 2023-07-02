macro_rules! integration_tests {
    ($($name:ident: $value:expr;)*) => {
        $(
            #[test]
            fn $name() {
                let (file, cycles, hash) = $value;
                let rom = File::new(file);
                let bus = Bus::new(rom, |_, _| {});
                let mut cpu = CPU::new(bus);
                cpu.reset();
                cpu.run(cycles);
                let actual = cpu.get_frame_hash();
                assert_eq!(actual, hash, "Actual hash was {}", actual);
            }
        )*
    }
}

mod tests {
    use nes::bus::Bus;
    use nes::cpu::CPU;
    use nes::ines_parser::File;

    integration_tests! {
        // CPU TESTS -------------------------------------------------------------------------------
        instr_test_v5: ("tests/instr_test-v5/all_instrs.nes", 71471530, 13190525789780138270);
        cpu_dummy_writes_oam: ("tests/cpu_dummy_writes/cpu_dummy_writes_oam.nes", 9706283, 1191060859770520958);
        cpu_dummy_writes_ppumem: ("tests/cpu_dummy_writes/cpu_dummy_writes_ppumem.nes", 6519753, 479349282414210491);
        cpu_exec_space_ppuio: ("tests/cpu_exec_space/test_cpu_exec_space_ppuio.nes", 1278383, 7085559936242306659);
        cpu_timing_tests: ("tests/cpu_timing_test6/cpu_timing_test.nes", 19081943, 11550658946518422994);

        // PPU TESTS -------------------------------------------------------------------------------
        sprite_ram: ("tests/blargg_ppu_tests_2005.09.15b/sprite_ram.nes", 980083, 3301376315147960416);
        vbl_clear_time: ("tests/blargg_ppu_tests_2005.09.15b/vbl_clear_time.nes", 1129709, 3301376315147960416);
        vram_access: ("tests/blargg_ppu_tests_2005.09.15b/vram_access.nes", 1010586, 3301376315147960416);
        ppu_vbl_nmi: ("tests/ppu_vbl_nmi/ppu_vbl_nmi.nes", 48063788, 1118392138081082278);

        sprite_hit_basics: ("tests/sprite_hit_tests_2005.10.05/01.basics.nes", 1365210, 4669044134520954011);
        sprite_hit_alignment: ("tests/sprite_hit_tests_2005.10.05/02.alignment.nes", 1305652, 4554223117083026616);
        sprite_hit_corners: ("tests/sprite_hit_tests_2005.10.05/03.corners.nes", 1007845, 16469816594957085986);
        sprite_hit_flip: ("tests/sprite_hit_tests_2005.10.05/04.flip.nes", 948283, 3369926367738944003);
        sprite_hit_left_clip: ("tests/sprite_hit_tests_2005.10.05/05.left_clip.nes", 1246092, 5036030515344815748);
        sprite_hit_right_edge: ("tests/sprite_hit_tests_2005.10.05/06.right_edge.nes", 1097186, 2434057420098902834);
        sprite_hit_screen_bottom: ("tests/sprite_hit_tests_2005.10.05/07.screen_bottom.nes", 1126970, 16983553457913087236);
        sprite_hit_double_height: ("tests/sprite_hit_tests_2005.10.05/08.double_height.nes", 1007846, 11903509375701802615);
        sprite_hit_timing_basics : ("tests/sprite_hit_tests_2005.10.05/09.timing_basics.nes", 2288413, 1686082719311973405);
        sprite_hit_timing_order : ("tests/sprite_hit_tests_2005.10.05/10.timing_order.nes", 2169294, 4393233932230211922);
        sprite_hit_edge_timing : ("tests/sprite_hit_tests_2005.10.05/11.edge_timing.nes", 2407540, 14313276901886322063);

        // MAPPER TESTS -------------------------------------------------------------------------------
        m0_p32k_c8k_v : ("tests/holy-mapperel/M0_P32K_C8K_V.nes", 148911, 3560412538058700980);
        m0_p32k_cr8k_v : ("tests/holy-mapperel/M0_P32K_CR8K_V.nes", 2263338, 5368662470135375084);
        m0_p32k_cr32k_v : ("tests/holy-mapperel/M0_P32K_CR32K_V.nes", 2263338,5368662470135375084);

        m1_p128k_c32k : ("tests/holy-mapperel/M1_P128K_C32K.nes", 178691, 14594562825770316186);
        m1_p128k_c32k_s8k : ("tests/holy-mapperel/M1_P128K_C32K_S8K.nes", 2431048, 6707839051576774982);
        m1_p128k_c32k_w8k : ("tests/holy-mapperel/M1_P128K_C32K_W8K.nes", 2431048, 6707839051576774982);
        m1_p128k_c128k : ("tests/holy-mapperel/M1_P128K_C32K.nes", 178691, 14594562825770316186);
        m1_p128k_c128k_s8k : ("tests/holy-mapperel/M1_P128K_C128K_S8K.nes", 2442022, 17191105395120435064);
        m1_p128k_c128k_w8k : ("tests/holy-mapperel/M1_P128K_C128K_W8K.nes", 2449514, 2543050301529417715);
        m1_p128k_cr8k: ("tests/holy-mapperel/M1_P128K_CR8K.nes", 2293119, 3560412538058700980);

        m3_p32k_c32k_h : ("tests/holy-mapperel/M3_P32K_C32K_H.nes", 148911, 7333994192358773729);
    }

    // CPU Tests -----------------------------------------------------------------------------------
    // let rom = File::new("tests/cpu_exec_space/test_cpu_exec_space_apu.nes"); // Fails - expected
    // let rom = File::new("tests/cpu_interrupts_v2/cpu_interrupts.nes"); // Fails - expected
    // let rom = File::new("tests/instr_misc/instr_misc.nes"); // Fails - expected
    // let rom = File::new("tests/instr_timing/instr_timing.nes"); // Fails - expected
    // let rom = File::new("tests/nestest/nestest.nes"); // Passes

    // PPU Tests -----------------------------------------------------------------------------------
    // let rom = File::new("tests/blargg_ppu_tests_2005.09.15b/palette_ram.nes"); // Fails - expected
    // let rom = File::new("tests/stress/NEStress.NES"); // ??
    // let rom = File::new("tests/scrolltest/scroll.nes"); // Passes
    // let rom = File::new("tests/scanline-a1/scanline.nes"); // Passes
    // let rom = File::new("tests/window5/colorwin_ntsc.nes"); // Passes
    // let rom = File::new("tests/spritecans-2011/spritecans.nes"); // Passes
}