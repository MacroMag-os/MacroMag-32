/*
 * MacroMag OS - Phase 3: Minimalist Value Product (MVP)
 * Copyright (C) 2026 Adrian Morawski / MacroMag Foundation
 * Licensed under GNU GPL v3.0
 *
 * Minimalistyczny interfejs graficzny (Framebuffer) oraz warstwa uruchomieniowa
 * działająca bezpośrednio nad okrojonym jądrem Linux (bez zbędnego GNU/X11/Wayland).
 */

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/utsname.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <linux/fb.h>

#define MACROMAG_VERSION "0.3.0-mvp"

typedef enum {
    EDITION_COMMUNITY_AIRGAPPED,
    EDITION_ENTERPRISE_CLOUD
} MacroMagEdition;

typedef struct {
    char kernel_release[64];
    char architecture[32];
    MacroMagEdition edition;
    int telemetry_enabled;
    int framebuffer_active;
} SystemContext;

/* Rysowanie minimalistycznego graficznego ekranu startowego bezpośrednio w /dev/fb0 */
int init_minimal_graphics(void) {
    int fb_fd = open("/dev/fb0", O_RDWR);
    if (fb_fd < 0) {
        /* Fallback dla środowiska bez ekranu (np. tryb szeregowy QEMU -nographic) */
        return 0;
    }

    struct fb_var_screeninfo vinfo;
    struct fb_fix_screeninfo finfo;

    if (ioctl(fb_fd, FBIOGET_FSCREENINFO, &finfo) < 0 ||
        ioctl(fb_fd, FBIOGET_VSCREENINFO, &vinfo) < 0) {
        close(fb_fd);
        return 0;
    }

    long screensize = vinfo.yres_virtual * finfo.line_length;
    uint8_t *fbp = (uint8_t *)mmap(0, screensize, PROT_READ | PROT_WRITE, MAP_SHARED, fb_fd, 0);

    if ((intptr_t)fbp != -1) {
        /* Wyczyszczenie ekranu głęboką czernią (0x0A0A0C) i narysowanie paska statusu MacroMag */
        memset(fbp, 0x0C, screensize);

        uint32_t bar_height = 24;
        if (vinfo.bits_per_pixel == 32 && vinfo.yres > bar_height) {
            for (uint32_t y = 0; y < bar_height; y++) {
                for (uint32_t x = 0; x < vinfo.xres; x++) {
                    long location = (x + vinfo.xoffset) * 4 + (y + vinfo.yoffset) * finfo.line_length;
                    *(fbp + location + 0) = 0xD4; /* Niebieski */
                    *(fbp + location + 1) = 0x8A; /* Zielony */
                    *(fbp + location + 2) = 0x1E; /* Czerwony */
                    *(fbp + location + 3) = 0x00;
                }
            }
        }
        munmap(fbp, screensize);
    }

    close(fb_fd);
    return 1;
}

/* Pobranie informacji o okrojonym jądrze Linux i konfiguracji edycji */
void init_system_context(SystemContext *ctx, int enterprise_mode) {
    struct utsname sys_info;
    if (uname(&sys_info) == 0) {
        snprintf(ctx->kernel_release, sizeof(ctx->kernel_release), "%s", sys_info.release);
        snprintf(ctx->architecture, sizeof(ctx->architecture), "%s", sys_info.machine);
    } else {
        strcpy(ctx->kernel_release, "linux-stripped");
        strcpy(ctx->architecture, "unknown");
    }

    if (enterprise_mode) {
        ctx->edition = EDITION_ENTERPRISE_CLOUD;
        ctx->telemetry_enabled = 0; /* Brak ukrytej telemetrii; jawny kanał zarządzania */
    } else {
        ctx->edition = EDITION_COMMUNITY_AIRGAPPED;
        ctx->telemetry_enabled = 0;
    }

    ctx->framebuffer_active = init_minimal_graphics();
}

/* Minimalistyczna pętla powłoki Virtualized Object Shell (VOS) */
void run_vos_minimal_loop(const SystemContext *ctx) {
    char input[256];

    printf("\n============================================================\n");
    printf(" MacroMag OS v%s | Linux-OS MVP Runtime\n", MACROMAG_VERSION);
    printf(" Kernel: %s | Arch: %s | GFX (/dev/fb0): %s\n",
           ctx->kernel_release,
           ctx->architecture,
           ctx->framebuffer_active ? "ACTIVE" : "HEADLESS");
    printf(" Edition: %s | Telemetry: %s\n",
           ctx->edition == EDITION_ENTERPRISE_CLOUD ? "ENTERPRISE (Cloud Control Plane)" : "COMMUNITY (Air-Gapped GPLv3)",
           ctx->telemetry_enabled ? "ON" : "ZERO-TELEMETRY");
    printf("============================================================\n");
    printf("Wpisz 'sys.info()', 'cloud.sync()' lub 'exit'.\n\n");

    while (1) {
        printf("vos://macromag> ");
        if (!fgets(input, sizeof(input), stdin)) {
            break;
        }
        input[strcspn(input, "\r\n")] = 0;

        if (strcmp(input, "exit") == 0) {
            printf("[VOS] Zamykanie sesji MacroMag OS...\n");
            break;
        } else if (strcmp(input, "sys.info()") == 0) {
            printf("{ \"os\": \"MacroMag-32\", \"phase\": 3, \"arch\": \"%s\", \"gfx\": %s }\n",
                   ctx->architecture,
                   ctx->framebuffer_active ? "true" : "false");
        } else if (strcmp(input, "cloud.sync()") == 0) {
            if (ctx->edition == EDITION_ENTERPRISE_CLOUD) {
                printf("[Enterprise] Nawiązywanie szyfrowanego połączenia VOS Object Pipeline...\n");
            } else {
                printf("[Community] Zablokowano: Tryb Air-Gapped nie zezwala na ruch sieciowy.\n");
            }
        } else if (strlen(input) > 0) {
            printf("[VOS] Nieznany obiekt/metoda: '%s'\n", input);
        }
    }
}

int main(int argc, char *argv[]) {
    int enterprise_mode = 0;
    if (argc > 1 && strcmp(argv[1], "--enterprise") == 0) {
        enterprise_mode = 1;
    }

    SystemContext ctx;
    init_system_context(&ctx, enterprise_mode);
    run_vos_minimal_loop(&ctx);

    return 0;
}
