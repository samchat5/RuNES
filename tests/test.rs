macro_rules! integration_tests {
    ($($name:ident: $value:expr;)*) => {
        $(
            #[test]
            fn $name() {
                let (file, frames, hash) = $value;
                let rom = File::new(file);
                let bus = Bus::new(&rom);
                let mut cpu = CPU::new(bus);
                cpu.reset();
                for _ in 0..frames {
                    cpu.run_until_frame();
                }
                let actual = cpu.get_frame_hash();
                assert_eq!(actual, hash, "Actual hash was {}", actual);
            }
        )*
    }
}

mod tests {
    use nes::core::bus::Bus;
    use nes::core::cpu::CPU;
    use nes::ines_parser::File;

    integration_tests! {
        // CPU TESTS -------------------------------------------------------------------------------
        instr_test_v5: ("tests/instr_test-v5/all_instrs.nes", 2398, 13190525789780138270);
        cpu_dummy_writes_oam: ("tests/cpu_dummy_writes/cpu_dummy_writes_oam.nes", 329, 18226267073703253929);
        cpu_dummy_writes_ppumem: ("tests/cpu_dummy_writes/cpu_dummy_writes_ppumem.nes", 234, 17557076518018075713);
        cpu_exec_space_ppuio: ("tests/cpu_exec_space/test_cpu_exec_space_ppuio.nes", 43, 7085559936242306659);
        cpu_timing_tests: ("tests/cpu_timing_test6/cpu_timing_test.nes", 612, 11550658946518422994);

        // PPU TESTS -------------------------------------------------------------------------------
        palette_ram: ("tests/blargg_ppu_tests_2005.09.15b/palette_ram.nes", 18, 3301376315147960416);
        sprite_ram: ("tests/blargg_ppu_tests_2005.09.15b/sprite_ram.nes", 18, 3301376315147960416);
        vbl_clear_time: ("tests/blargg_ppu_tests_2005.09.15b/vbl_clear_time.nes", 24, 3301376315147960416);
        vram_access: ("tests/blargg_ppu_tests_2005.09.15b/vram_access.nes", 19, 3301376315147960416);
        ppu_vbl_nmi: ("tests/ppu_vbl_nmi/ppu_vbl_nmi.nes", 1624, 3000831971158866996);
        ppu_read_buffer: ("tests/ppu_read_buffer/test_ppu_read_buffer.nes", 1269, 10957719060148031592);
        oam_stress: ("tests/oam_stress/oam_stress.nes", 1703, 60536158850127617);

        sprite_hit_basics: ("tests/sprite_hit_tests_2005.10.05/01.basics.nes", 32, 4669044134520954011);
        sprite_hit_alignment: ("tests/sprite_hit_tests_2005.10.05/02.alignment.nes", 31, 4554223117083026616);
        sprite_hit_corners: ("tests/sprite_hit_tests_2005.10.05/03.corners.nes", 22, 16469816594957085986);
        sprite_hit_flip: ("tests/sprite_hit_tests_2005.10.05/04.flip.nes", 19, 3369926367738944003);
        sprite_hit_left_clip: ("tests/sprite_hit_tests_2005.10.05/05.left_clip.nes", 30, 5036030515344815748);
        sprite_hit_right_edge: ("tests/sprite_hit_tests_2005.10.05/06.right_edge.nes", 23, 2434057420098902834);
        sprite_hit_screen_bottom: ("tests/sprite_hit_tests_2005.10.05/07.screen_bottom.nes", 24, 16983553457913087236);
        sprite_hit_double_height: ("tests/sprite_hit_tests_2005.10.05/08.double_height.nes", 20, 11903509375701802615);
        sprite_hit_timing_basics: ("tests/sprite_hit_tests_2005.10.05/09.timing_basics.nes", 66, 1686082719311973405);
        sprite_hit_timing_order: ("tests/sprite_hit_tests_2005.10.05/10.timing_order.nes", 66, 4393233932230211922);
        sprite_hit_edge_timing: ("tests/sprite_hit_tests_2005.10.05/11.edge_timing.nes", 79, 14313276901886322063);

        // MAPPER TESTS -------------------------------------------------------------------------------
        m0_p32k_c8k_v: ("tests/holy-mapperel/M0_P32K_C8K_V.nes", 6, 16402098814799907941);
        m0_p32k_cr8k_v: ("tests/holy-mapperel/M0_P32K_CR8K_V.nes", 77, 12779805597582904188);
        m0_p32k_cr32k_v: ("tests/holy-mapperel/M0_P32K_CR32K_V.nes", 77, 12779805597582904188);

        m1_p128k_c32k: ("tests/holy-mapperel/M1_P128K_C32K.nes", 7, 3626736649374408985);
        m1_p128k_c32k_s8k: ("tests/holy-mapperel/M1_P128K_C32K_S8K.nes", 82, 14698170090460665526);
        m1_p128k_c32k_w8k: ("tests/holy-mapperel/M1_P128K_C32K_W8K.nes", 83, 14698170090460665526);
        m1_p128k_c128k: ("tests/holy-mapperel/M1_P128K_C128K.nes", 7, 9007347242850485333);
        m1_p128k_c128k_s8k: ("tests/holy-mapperel/M1_P128K_C128K_S8K.nes", 83, 1836053688703264546);
        m1_p128k_c128k_w8k: ("tests/holy-mapperel/M1_P128K_C128K_W8K.nes", 83, 1836053688703264546);
        m1_p128k_cr8k: ("tests/holy-mapperel/M1_P128K_CR8K.nes", 78, 993067101538369690);

        m3_p32k_c32k_h: ("tests/holy-mapperel/M3_P32K_C32K_H.nes", 6, 12112331729405102634);

        // APU TESTS -------------------------------------------------------------------------------
        len_ctr: ("tests/blargg_apu_2005.07.30/01.len_ctr.nes", 26, 3301376315147960416);
        len_table: ("tests/blargg_apu_2005.07.30/02.len_table.nes", 12, 3301376315147960416);
        irq_flag: ("tests/blargg_apu_2005.07.30/03.irq_flag.nes", 17, 3301376315147960416);
        clock_jitter: ("tests/blargg_apu_2005.07.30/04.clock_jitter.nes", 17, 3301376315147960416);
        len_timing_mode0: ("tests/blargg_apu_2005.07.30/05.len_timing_mode0.nes", 22, 3301376315147960416);
        len_timing_mode1: ("tests/blargg_apu_2005.07.30/06.len_timing_mode1.nes", 24, 3301376315147960416);
        irq_flag_timing: ("tests/blargg_apu_2005.07.30/07.irq_flag_timing.nes", 18, 3301376315147960416);
        irq_timing: ("tests/blargg_apu_2005.07.30/08.irq_timing.nes", 17, 3301376315147960416);
        reset_timing: ("tests/blargg_apu_2005.07.30/09.reset_timing.nes", 11, 3301376315147960416);
        len_halt_timing: ("tests/blargg_apu_2005.07.30/10.len_halt_timing.nes", 16, 3301376315147960416);
        len_reload_timing: ("tests/blargg_apu_2005.07.30/11.len_reload_timing.nes", 17, 3301376315147960416);
    }

    // CPU Tests -----------------------------------------------------------------------------------
    // let rom = File::new("tests/cpu_exec_space/test_cpu_exec_space_apu.nes"); // Fails - expected
    // let rom = File::new("tests/cpu_interrupts_v2/cpu_interrupts.nes"); // Fails - expected
    // let rom = File::new("tests/instr_misc/instr_misc.nes"); // Fails - expected
    // let rom = File::new("tests/instr_timing/instr_timing.nes"); // Fails - expected
    // let rom = File::new("tests/nestest/nestest.nes"); // Passes

    // PPU Tests -----------------------------------------------------------------------------------
    // let rom = File::new("tests/stress/NEStress.NES"); // ??
    // let rom = File::new("tests/scrolltest/scroll.nes"); // Passes
    // let rom = File::new("tests/scanline-a1/scanline.nes"); // Passes
    // let rom = File::new("tests/window5/colorwin_ntsc.nes"); // Passes
    // let rom = File::new("tests/spritecans-2011/spritecans.nes"); // Passes
    // let rom = File::new("tests/nmi_sync/demo_ntsc.nes"); // Fails

    // Mapper Tests --------------------------------------------------------------------------------
    // m1_p512k_cr8k_s8k: ("tests/holy-mapperel/M1_P512K_CR8K_S8K.nes", ?, ?); // Fails - expected
    // m1_p512k_cr8k_w8k: ("tests/holy-mapperel/M1_P512K_CR8K_W8K.nes", ?, ?); // Fails - expected
}
