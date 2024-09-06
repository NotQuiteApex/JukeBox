// JukeBox Case

/* [General settings] */
// Generates top case piece
gen_top = false;
// Generates bottom case piece
gen_bot = false;
// Generates leg case piece
gen_leg = false;
// Generates screen case piece
gen_scr = false;
// Generates screen detail for top case piece
gen_detail = false;
// Case size (width & height)
cS = 98;
// Case corner radius (rounded corners)
cR = 3;
// Case mounting hardware offset
cmO = 7;
// Case mounting hardware bolt size
cmB = 3.5;
// Case mounting hardware nut size (center to corner)
cmN = 3.3;
// Face count on rounded objects
$fn=32;

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
cpH = 3;
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

/* [Case top settings] */
// Case top height
ctH = 8;
// Case top wall size
ctW = 2.5;
// Case top mounting plate size
ctM = 9;
// Case top mounting plate height
ctMH = 3;

/* [Logo settings] */
// TODO
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

// https://www.youtube.com/watch?v=gKOkJWiTgAY
module roundedsquare(xdim, ydim, zdim, rdim){
    hull() {
        translate([     rdim,      rdim, 0]) cylinder(h=zdim, r=rdim);
        translate([xdim-rdim,      rdim, 0]) cylinder(h=zdim, r=rdim);
        translate([     rdim, ydim-rdim, 0]) cylinder(h=zdim, r=rdim);
        translate([xdim-rdim, ydim-rdim, 0]) cylinder(h=zdim, r=rdim);
    }
}
module chamferedsquare(xdim, ydim, zdim, r1dim, r2dim){
    bigrdim = max(r1dim, r2dim);
    hull() {
        translate([     bigrdim,      bigrdim, 0]) cylinder(h=zdim, r1=r1dim, r2=r2dim);
        translate([xdim-bigrdim,      bigrdim, 0]) cylinder(h=zdim, r1=r1dim, r2=r2dim);
        translate([     bigrdim, ydim-bigrdim, 0]) cylinder(h=zdim, r1=r1dim, r2=r2dim);
        translate([xdim-bigrdim, ydim-bigrdim, 0]) cylinder(h=zdim, r1=r1dim, r2=r2dim);
    }
}

module speaker_icon() {
    scale([5/6, 5/6, 1]) {
        difference() {
            cylinder(r=7, h=1, center=true);
            cylinder(r=6, h=1, center=true);
        }
        cube([1, 13, 1], center=true);
        translate([ 4, 0, 0]) cube([1, 9, 1], center=true);
        translate([-4, 0, 0]) cube([1, 9, 1], center=true);
        translate([0, -2.25, 0]) cube([11, 1, 1], center=true);
        translate([0,  2.25, 0]) cube([11, 1, 1], center=true);
    }
}

SOX = cS / 2 - (csSCRW + ctW * 2) / 2; // screen origin x
SOY = cS - 7.5 - ctW - csSCRH / 2; // screen origin y

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
                    translate([-1, 70.5-0.5, -1]) cube([ctW+2, 10.5+1, ctH+1]);
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
            // Mounting plates
            translate([         0,          0, ctH-ctMH]) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);
            translate([cS-ctM-ctW,          0, ctH-ctMH]) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);
            translate([         0, cS-ctM-ctW, ctH-ctMH]) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);
            translate([cS-ctM-ctW, cS-ctM-ctW, ctH-ctMH]) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);

            // Support poles (keyboard)
            // Left
            translate([kbX-(kbSW-kbS)/2-kbSH, kbY-(kbSH-kbS)/2-  kbSH/2, ctH-ctMH]) cube([kbSW-kbS, kbSH-kbS, ctMH]);
            translate([kbX-(kbSW-kbS)/2-kbSH, kbY-(kbSH-kbS)/2+  kbSH/2, ctH-ctMH]) cube([kbSW-kbS, kbSH-kbS, ctMH]);
            translate([kbX-(kbSW-kbS)/2-kbSH, kbY-(kbSH-kbS)/2+3*kbSH/2, ctH-ctMH]) cube([kbSW-kbS, kbSH-kbS, ctMH]);
            // Mid
            translate([kbX-(kbSW-kbS)/2, kbY-(kbSH-kbS)/2-  kbSH/2, ctH-ctMH]) cube([kbSW-kbS, kbSH-kbS, ctMH]);
            translate([kbX-(kbSW-kbS)/2, kbY-(kbSH-kbS)/2+  kbSH/2, ctH-ctMH]) cube([kbSW-kbS, kbSH-kbS, ctMH]);
            translate([kbX-(kbSW-kbS)/2, kbY-(kbSH-kbS)/2+3*kbSH/2, ctH-ctMH]) cube([kbSW-kbS, kbSH-kbS, ctMH]);
            // Right
            translate([kbX-(kbSW-kbS)/2+kbSH, kbY-(kbSH-kbS)/2-  kbSH/2, ctH-ctMH]) cube([kbSW-kbS, kbSH-kbS, ctMH]);
            translate([kbX-(kbSW-kbS)/2+kbSH, kbY-(kbSH-kbS)/2+  kbSH/2, ctH-ctMH]) cube([kbSW-kbS, kbSH-kbS, ctMH]);
            translate([kbX-(kbSW-kbS)/2+kbSH, kbY-(kbSH-kbS)/2+3*kbSH/2, ctH-ctMH]) cube([kbSW-kbS, kbSH-kbS, ctMH]);
        }

        union() {
            // keyboard key holes
            for (w=[0:kbW-1]) {
                for (h=[0:kbH-1]) {
                    translate([kbX-kbOX+w*kbSW, kbY-kbOY+h*kbSH, ctH+1]) cube([kbS, kbS, 2], center=true);
                }
            }

            // mounting hardware holes
            translate([   cmO,    cmO, ctH-4]) { cylinder(d=cmB, h=5); translate([0,0,2]) cylinder(d2=6, d1=2, h=3); }
            translate([cS-cmO,    cmO, ctH-4]) { cylinder(d=cmB, h=5); translate([0,0,2]) cylinder(d2=6, d1=2, h=3); }
            translate([   cmO, cS-cmO, ctH-4]) { cylinder(d=cmB, h=5); translate([0,0,2]) cylinder(d2=6, d1=2, h=3); }
            translate([cS-cmO, cS-cmO, ctH-4]) { cylinder(d=cmB, h=5); translate([0,0,2]) cylinder(d2=6, d1=2, h=3); }
            
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

            // Screen floor
            if (gen_scr) translate([SOX, SOY, 0]) union() {
                chamferedsquare(csSCRW + ctW * 2, csSCRH + ctW * 2, clC, cR-clC, cR);
                translate([0, 0, clC]) roundedsquare(csSCRW + ctW * 2, csSCRH + ctW * 2, clH-clC, cR);
                translate([clS, clS+(csSCRH-16), clH]) {
                    roundedsquare(csSCRW-1, 15, clH, cR);
                    translate([0, 4, clH]) roundedsquare(csSCRW-1, 11, 1.6, cR);
                }
            }

            // PCB table
            difference() {
                translate([    clS,     clS, clH]) roundedsquare(cS-clS*2, cS-clS*2, cpH, cpR);
                translate([clS+cpW, clS+cpW, clH]) cube([cS-clS*2-cpW*2, cS-clS*2-cpW*2, cpH]);
            }
            translate([           clS,            clS, clH]) roundedsquare(cpM, cpM, cpH, cpR);
            translate([cS-clS*2-cpW*2,            clS, clH]) roundedsquare(cpM, cpM, cpH, cpR);
            translate([           clS, cS-clS*2-cpW*2, clH]) roundedsquare(cpM, cpM, cpH, cpR);
            translate([cS-clS*2-cpW*2, cS-clS*2-cpW*2, clH]) roundedsquare(cpM, cpM, cpH, cpR);

            // USB-C pillar
            translate([0, 70.25, clH]) cube([clS, 11, cpW+1.6]);
        }

        union() {
            // Bolt holes
            translate([   cmO,    cmO, 0]) cylinder(d=cmB, h=7);
            translate([cS-cmO,    cmO, 0]) cylinder(d=cmB, h=7);
            translate([   cmO, cS-cmO, 0]) cylinder(d=cmB, h=7);
            translate([cS-cmO, cS-cmO, 0]) cylinder(d=cmB, h=7);

            // Nut holes
            translate([   cmO,    cmO, 0]) cylinder($fn=6, r=cmN, h=2.375);
            translate([cS-cmO,    cmO, 0]) cylinder($fn=6, r=cmN, h=2.375);
            translate([   cmO, cS-cmO, 0]) cylinder($fn=6, r=cmN, h=2.375);
            translate([cS-cmO, cS-cmO, 0]) cylinder($fn=6, r=cmN, h=2.375);

            // Hole for screen cable
            // translate([cS/2, cS-clS-cpW/2, clH+cpH/2]) cube([24, cpW, cpH], center=true);

            // Indents for rubber feet
            translate([   cpF,    cpF, 0]) cylinder(d=cpFD, h=cpFH);
            translate([cS-cpF,    cpF, 0]) cylinder(d=cpFD, h=cpFH);
            translate([   cpF, cS-cpF, 0]) cylinder(d=cpFD, h=cpFH);
            translate([cS-cpF, cS-cpF, 0]) cylinder(d=cpFD, h=cpFH);
        }
    }
}

module case_detail() {
    if (!gen_scr) translate([logoX, logoY, ctH+0.75]) linear_extrude(height=0.5, center=true) scale([logoS, logoS, 0.5]) import(file="../assets/textlogo.svg", center=true);
    translate([cS/2-37, cS-18, ctH+0.75]) scale([1.1, 1.1, 0.5]) speaker_icon();
    translate([cS/2+37, cS-18, ctH+0.75]) scale([1.1, 1.1, 0.5]) speaker_icon();
    // translate([cS/2, 6, ctH+1]) linear_extrude(height=1, center=true) text("friendteam.biz", size=4, halign="center", valign="center", font="Cascadia Mono:style=Regular");
}

module case_leg() {
    lH = clH+ctH+1;
    lS = cS;
    difference() {
        union() {
            translate([ -clipR,  0, 0]) cube([clipR, lH, clipW]);
            translate([     lS,  0, 0]) cube([clipR, lH, clipW]);
            translate([      0, lH, 0]) cube([lS, clipR, clipW]);

            translate([ 0,  0, 0]) cylinder(h=clipW, r=clipR);
            translate([lS,  0, 0]) cylinder(h=clipW, r=clipR);
            translate([ 0, lH, 0]) cylinder(h=clipW, r=clipR);
            translate([lS, lH, 0]) cylinder(h=clipW, r=clipR);

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

if (gen_top) translate([0, 0, clH]) color([1, 1, 0]) case_top();
if (gen_bot) color([0, 1, 0]) case_bottom();
if (gen_leg) color([1, 0, 0]) case_leg();
if (gen_detail) translate([0, 0, clH]) color([0, 0, 1]) case_detail();
