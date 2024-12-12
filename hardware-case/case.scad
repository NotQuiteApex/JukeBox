// JukeBox Case

/* [General settings] */
// Generates top case piece
gen_top = false;
// Generates bottom case piece
gen_bot = false;
// Generates leg case piece
gen_leg = false;
// Generates screen case piece
gen_scr = true;
// Generates screen detail for top case piece
gen_detail = false;
// Case size (width & height)
cS = 98;
// Case corner radius (rounded corners)
cR = 3;
// Case mounting hardware offset
cmO = 7.5;
// Case mounting hardware bolt size
cmB = 3.5;
// Case mounting hardware nut size (center to corner)
cmN = 3.3;
// Face count on rounded objects
$fn=32;
// Case connector hole size
ioHS = 11;
// Case connector hole offset
ioHO = cS-cmO-7;
// Case connector hole buffer
ioHB = 0.125;

/* [Case top settings] */
// Case top height
ctH = 8;
// Case top wall size
ctW = 2.5;
// Case top mounting plate size
ctM = 9;
// Case top mounting plate height
ctMH = 3.4;

/* [Case bottom settings] */
// Case bottom lip height
clH = 3;
// Case bottom lip chamfer size
clC = 1;
// Case bottom lip size
clS = 3;
// Case bottom PCB corner radius (rounded corners)
cpR = 3;
// Case bottom PCB height
cpH = 2.6;
// Case bottom PCB wall size
cpW = 3;
// Case bottom PCB mounting plate size
cpM = 9;
// Case bottom rubber feet spot offset
cpF = 20;
// Case bottom rubber feet spot diameter
cpFD = 10;
// Case bottom rubber feet spot depth
cpFH = 2;

/* [Logo settings] */
// Logo size scale
logoS = 0.035;
// Logo position (X)
logoX = 49;
// Logo position (Y)
logoY = 83;

/* [Keyboard settings] */
// Keyboard key size
kbS = 17;
// Keyboard key matrix position (X)
kbX = 49;
// Keyboard key matrix position (Y)
kbY = 39;
// Keyboard key matrix offset (X)
kbOX = 30;
// Keyboard key matrix offset (Y)
kbOY = 20;
// Keyboard key matrix width
kbW = 4;
// Keyboard key matrix height
kbH = 3;
// Keyboard key matrix spacing width
kbSW = 20;
// Keyboard key matrix spacing height
kbSH = 20;

/* [Case leg settings] */
// Leg width
clipW = 10;
// Leg rounding radius
clipR = 4;

/* [Case screen settings] */
// Screen width
csSCRW = 48;
// Screen height
csSCRH = 34.8;

module rect4(x1, y1, x2, y2, h) {
    translate([x1, y1, h]) children();
    translate([x2, y1, h]) children();
    translate([x1, y2, h]) children();
    translate([x2, y2, h]) children();
}
module square4(s1, s2, h) {
    rect4(s1, s1, s2, s2, h) children();
}
// https://www.youtube.com/watch?v=gKOkJWiTgAY
module roundedsquare(xdim, ydim, zdim, rdim){
    hull() {
        rect4(rdim, rdim, xdim-rdim, ydim-rdim, 0) cylinder(h=zdim, r=rdim);
    }
}
module chamferedsquare(xdim, ydim, zdim, r1dim, r2dim){
    bigrdim = max(r1dim, r2dim);
    hull() {
        rect4(bigrdim, bigrdim, xdim-bigrdim, ydim-bigrdim, 0) cylinder(h=zdim, r1=r1dim, r2=r2dim);
    }
}

module speaker_icon() {
    scale([5/6, 5/6, 1]) {
        difference() {
            cylinder(r=7, h=1, center=true);
            cylinder(r=6, h=1, center=true);
        }
        cube([1, 13, 1], center=true);
        translate([ 4, 0, 0]) cube([1, 10, 1], center=true);
        translate([-4, 0, 0]) cube([1, 10, 1], center=true);
        translate([0, -2.25, 0]) cube([12, 1, 1], center=true);
        translate([0,  2.25, 0]) cube([12, 1, 1], center=true);
    }
}

SOX = cS / 2 - (csSCRW + ctW * 2) / 2; // screen origin x
SOY = cS - 8.5 - ctW - csSCRH / 2; // screen origin y

module case_top() {
    difference() {
        union() {
            difference() {
                union() {
                    // Top shell body
                    translate([0, 0, ctH]) chamferedsquare(cS, cS, 1, 3, 2);
                    roundedsquare(cS, cS, ctH, cR);
                    // Screen body
                    if (gen_scr) translate([SOX, SOY, 0]) union() {
                        sW = csSCRW + ctW * 2;
                        sH = csSCRH + ctW * 2;
                        translate([0, 0, ctH]) chamferedsquare(sW, sH, 1, 3, 2);
                        roundedsquare(sW, sH, ctH, 3);
                    }
                }
                union() {
                    // USB-C hole
                    translate([-1, ioHO-ioHS, -1]) cube([ctW+2, ioHS, ctH+1]);
                    // Interior
                    translate([ctW, ctW, -1]) roundedsquare(cS-ctW*2, cS-ctW*2, ctH+1, cR);
                    // Screen cutout
                    if (gen_scr) translate([SOX, SOY, 0]) union() {
                        // Cutout interior
                        translate([ctW, ctW, 0]) cube([csSCRW, csSCRH, ctH]);
                        // Screen cutout
                        translate([ctW + 2, ctW + 2, ctH]) cube([csSCRW - 3.5, csSCRH - 4, 1]);
                    }
                }
            }

            h = ctH-ctMH;

            // Mounting plates
            square4(0, cS-ctM-ctW, h) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);

            // Support poles (keyboard)
            kS2 = kbSH / 2;
            kSW = kbSW - kbS;
            kSH = kbSH - kbS;
            kSW2 = kSW / 2;
            kSH2 = kSH / 2;
            kXW2 = kbX - kSW2;
            kYH2 = kbY - kSH2;
            c = [kSW, kSH, ctMH];
            for (i = [-1:1]) {
                for (j = [-1:2:3]) {
                    translate([kXW2 + kbSH * i, kYH2 + kS2 * j, h]) cube(c);
                }
            }
        }

        union() {
            // keyboard key holes
            kX = kbX-kbOX;
            kY = kbY-kbOY;
            ch = ctH + 1;
            for (w=[0:kbW-1]) {
                for (h=[0:kbH-1]) {
                    translate([kX + kbSW * w, kY + kbSH * h, ch]) cube([kbS, kbS, 2], center=true);
                }
            }

            // mounting hardware holes
            square4(cmO, cS - cmO, ctH - 4) {
                cylinder(d=cmB, h=5);
                translate([0, 0, 2]) cylinder(d2=6, d1=2, h=3);
            }
            
            // Jukebox logo
            if (gen_detail) case_detail();
        }
    }
}

module case_bottom() {
    difference() {
        union() {
            // Case floor
            chamferedsquare(cS, cS, clC, cR-clC, cR);
            translate([0, 0, clC]) roundedsquare(cS, cS, clH-clC, cR);

            // Screen table
            if (gen_scr) translate([SOX, SOY, 0]) union() {
                chamferedsquare(csSCRW + ctW * 2, csSCRH + ctW * 2, clC, cR-clC, cR);
                translate([0, 0, clC]) roundedsquare(csSCRW + ctW * 2, csSCRH + ctW * 2, clH-clC, cR);
                translate([clS, clS+(csSCRH-16), clH]) {
                    translate([0, 1, 0]) roundedsquare(csSCRW-1, 14, cpH, cR);
                    translate([0, 4, cpH]) roundedsquare(csSCRW-1, 11, 1.6, cR);
                }
            }

            // PCB table
            difference() {
                translate([    clS,     clS, clH]) roundedsquare(cS-clS*2, cS-clS*2, cpH, cpR);
                translate([clS+cpW, clS+cpW, clH]) cube([cS-clS*2-cpW*2, cS-clS*2-cpW*2, cpH]);
            }
            square4(clS, cS-clS*2-cpW*2, clH) roundedsquare(cpM, cpM, cpH, cpR);

            // USB-C pillar
            translate([0, ioHO-ioHS+ioHB, clH]) cube([1.5, ioHS-ioHB*2, 4]);
        }

        union() {
            cmO2 = cS-cmO;
            nH = 2.375;
            square4(cmO, cmO2, 0) cylinder(d=cmB, h=7);
            square4(cmO, cmO2, 0) cylinder($fn=6, r=cmN, h=nH);
            square4(cmO, cmO2, nH) cylinder($fn=6, r1=cmN, r2=cmB/2, h=1);

            // Indents for rubber feet
            square4(cpF, cS-cpF, 0) cylinder(d=cpFD, h=cpFH/2);
            square4(cpF, cS-cpF, cpFH/2) cylinder(d1=cpFD, d2=cpFD-2, h=cpFH/2);

            // cutout for through hole components
            translate([clS, ioHO-ioHS+ioHB, clH]) cube([10, ioHS-ioHB*2, cpH]);
        }
    }
}

module case_detail() {
    x1 = cS / 2 - 37;
    x2 = cS / 2 + 37;
    y = cS - 18;
    h = ctH + 0.75;
    if (!gen_scr)
        translate([logoX, logoY, h])
            linear_extrude(height=0.5, center=true)
                scale([logoS, logoS, 0.5])
                    import(file="../assets/textlogo.svg", center=true);
    translate([x1, y, h]) scale([1.1, 1.1, 0.5]) speaker_icon();
    translate([x2, y, h]) scale([1.1, 1.1, 0.5]) speaker_icon();
}

module case_leg() {
    lH = clH+ctH+1;
    lS = cS;
    difference() {
        union() {
            translate([ -clipR,  0, 0]) cube([clipR, lH, clipW]);
            translate([     lS,  0, 0]) cube([clipR, lH, clipW]);
            translate([      0, lH, 0]) cube([lS, clipR, clipW]);
            
            rect4(0, 0, lS, lH, 0) cylinder(h=clipW, r=clipR);

            hull() {
                translate([   0,      lH, 0]) cylinder(h=clipW, r=clipR);
                translate([  lS,      lH, 0]) cylinder(h=clipW, r=clipR);
                translate([lS/3, lH+lS/2, 0]) cylinder(h=clipW, r=clipR);
            }
        }
        union() {
            translate([0,0,-1]) cube([lS, lH, clipW+2]);
            translate([0,0,-1]) linear_extrude(height=clipW+2) polygon(points=[[0,lH],[lS,lH],[lS/3, lH+lS/2]]);

            // rubber feet spots (angling is estimated, TOFIX?)
            stop1=0.1;
            stop2=0.9;
            translate([lS/3+stop1*(2*lS/3), lH+lS/2-stop1*(lS/2), 0]) rotate([0,90,53.125]) translate([-clipW/2,0,cpFH]) cylinder(d=cpFD, h=cpFH);
            translate([lS/3+stop2*(2*lS/3), lH+lS/2-stop2*(lS/2), 0]) rotate([0,90,53.125]) translate([-clipW/2,0,cpFH]) cylinder(d=cpFD, h=cpFH);
        }
    }
    translate([0, lH, 0]) cube([lS, clipR, clipW]);
}

difference() {
    union() {
        if (gen_top) translate([0, 0, clH]) color([1, 1, 0]) case_top();
        if (gen_bot) color([0, 1, 0]) case_bottom();
    }
    
    // cutout for io
    union() {
        // USB side
        // translate([0, ioHO-ioHS+ioHB, clH]) cube([clS, ioHS-ioHB*2, cpW+1.6]);
        // translate([0, ioHO-ioHS, clH]) cube([3, ioHS, 10]);

        // Debug pad side
        ioHS2 = 23-3;
        ioHO2 = cS-cmO-2.5-1.5;
        translate([cS-2.5, ioHO2-ioHS2, clH]) cube([2.5, ioHS2, 3.8]);
        translate([cS-12, ioHO2-ioHS2, clH]) cube([12, ioHS2, 3]);
    }
}
if (gen_leg) color([1, 0, 0]) case_leg();
// if (gen_detail) translate([0, 0, clH]) color([0, 0, 1]) case_detail();
