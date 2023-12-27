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
// Case size (width & height)
cS = 98;
// Case corner radius (rounded corners)
cR = 3;
// Case mounting hardware offset
cmO = 7;
// Case mounting hardware bolt size
cmB = 3.5;
// Case mounting hardware nut size (corner to corner)
cmN = 3.5;
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
csSW = 66;
csSH = 41;
csSCRW = 49;
csSCRH = 35;
csSCRM = 2;

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

module case_bottom() {
    difference() {
        union() {
            // Case floor
            chamferedsquare(cS, cS, clC, cR-clC, cR);
            translate([0, 0, clC]) roundedsquare(cS, cS, clH-clC, cR);

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
            translate([0, 70.5, clH]) cube([clS, 10.5, cpW+1.6]);
        }

        union() {
            // Bolt holes
            translate([   cmO,    cmO, 0]) cylinder(d=cmB, h=7);
            translate([cS-cmO,    cmO, 0]) cylinder(d=cmB, h=7);
            translate([   cmO, cS-cmO, 0]) cylinder(d=cmB, h=7);
            translate([cS-cmO, cS-cmO, 0]) cylinder(d=cmB, h=7);

            // Nut holes
            translate([   cmO,    cmO, 0]) cylinder($fn=6, r=cmN, h=2.5);
            translate([cS-cmO,    cmO, 0]) cylinder($fn=6, r=cmN, h=2.5);
            translate([   cmO, cS-cmO, 0]) cylinder($fn=6, r=cmN, h=2.5);
            translate([cS-cmO, cS-cmO, 0]) cylinder($fn=6, r=cmN, h=2.5);

            // Hole for screen cable
            translate([cS/2, cS-clS-cpW/2, clH+cpH/2]) cube([24, cpW, cpH], center=true);

            // Indents for rubber feet
            translate([   cpF,    cpF, 0]) cylinder(d=cpFD, h=cpFH);
            translate([cS-cpF,    cpF, 0]) cylinder(d=cpFD, h=cpFH);
            translate([   cpF, cS-cpF, 0]) cylinder(d=cpFD, h=cpFH);
            translate([cS-cpF, cS-cpF, 0]) cylinder(d=cpFD, h=cpFH);
        }
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

module case_screen() {
    difference() {
        union() {
            translate([0, 0, ctH]) chamferedsquare(csSW, csSH, 1, 3, 2);
            roundedsquare(csSW, csSH, ctH, 3);
        }
        union() {
            // Cutout interior
            translate([ctW, ctW, -1]) cube([csSW-ctW*2, csSH-ctW*2, ctH+1]);
            translate([ctW, -cR, -1]) cube([csSW-ctW*2, cR*3, ctH+1]);

            // Screen cutout
            translate([csSW/2, csSH/2+0.5, 0]) cube([csSCRW, csSCRH, 3*ctH], center=true);

            // Mounting hole cutout
            translate([csSW/2-53/2, csSH/2+0.5-29/2, 0]) cylinder(d=csSCRM, h=3*ctH);
            translate([csSW/2+53/2, csSH/2+0.5-29/2, 0]) cylinder(d=csSCRM, h=3*ctH);
            translate([csSW/2-53/2, csSH/2+0.5+29/2, 0]) cylinder(d=csSCRM, h=3*ctH);
            translate([csSW/2+53/2, csSH/2+0.5+29/2, 0]) cylinder(d=csSCRM, h=3*ctH);
        }
    }
}

module case_top() {
    difference() {
        union() {
            difference() {
                union() {
                    // top shell
                    translate([0, 0, ctH]) chamferedsquare(cS, cS, 1, 3, 2);
                    roundedsquare(cS, cS, ctH, cR);
                    if (gen_scr) translate([cS/2-csSW/2, cS-3.5, 0]) case_screen();
                }
                union() {
                    // USB-C hole
                    translate([-1, 70.5-0.5, -1]) cube([ctW+2, 10.5+1, ctH+1]);
                    // Interior
                    translate([ctW, ctW, -1]) roundedsquare(cS-ctW*2, cS-ctW*2, ctH+1, cR);
                    // Screen cutout
                    if (gen_scr) translate([cS/2-csSW/2+ctW, cS-4, 0]) cube([csSW-2*ctW, 2*ctW, ctH]);
                }
            }
            // Mounting plates
            translate([         0,          0, ctH-ctMH]) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);
            translate([cS-ctM-ctW,          0, ctH-ctMH]) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);
            translate([         0, cS-ctM-ctW, ctH-ctMH]) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);
            translate([cS-ctM-ctW, cS-ctM-ctW, ctH-ctMH]) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);

            // Support poles (keyboard)
            translate([kbX-(kbSW-kbS)/2, kbY-(kbSH-kbS)/2-  kbSH/2, ctH-ctMH]) cube([kbSW-kbS, kbSH-kbS, ctMH]);
            translate([kbX-(kbSW-kbS)/2, kbY-(kbSH-kbS)/2+  kbSH/2, ctH-ctMH]) cube([kbSW-kbS, kbSH-kbS, ctMH]);
            translate([kbX-(kbSW-kbS)/2, kbY-(kbSH-kbS)/2+3*kbSH/2, ctH-ctMH]) cube([kbSW-kbS, kbSH-kbS, ctMH]);
        }

        union() {
            // keyboard key holes
            for (w=[0:kbW-1]) {
                for (h=[0:kbH-1]) {
                    translate([kbX-kbOX+w*kbSW, kbY-kbOY+h*kbSH, ctH+1]) cube([kbS, kbS, 2], center=true);
                }
            }

            // mounting hardware holes
            translate([   cmO,    cmO, ctH-4]) { cylinder(d=cmB, h=5); translate([0,0,2]) cylinder(d2=7, d1=3, h=3); }
            translate([cS-cmO,    cmO, ctH-4]) { cylinder(d=cmB, h=5); translate([0,0,2]) cylinder(d2=7, d1=3, h=3); }
            translate([   cmO, cS-cmO, ctH-4]) { cylinder(d=cmB, h=5); translate([0,0,2]) cylinder(d2=7, d1=3, h=3); }
            translate([cS-cmO, cS-cmO, ctH-4]) { cylinder(d=cmB, h=5); translate([0,0,2]) cylinder(d2=7, d1=3, h=3); }
            
            // Jukebox logo
            translate([logoX, logoY, ctH+1]) linear_extrude(height=1, center=true) scale([logoS, logoS, 1]) import(file="../assets/textlogo.svg", center=true);
            translate([cS/2-37, cS-18, ctH+1]) scale([1.1, 1.1, 1]) speaker_icon();
            translate([cS/2+37, cS-18, ctH+1]) scale([1.1, 1.1, 1]) speaker_icon();
            // translate([cS/2, 6, ctH+1]) linear_extrude(height=1, center=true) text("friendteam.biz", size=4, halign="center", valign="center", font="Cascadia Mono:style=Regular");
        }
    }
}

module case_leg() {
    lH = clH+ctH+1.5;
    lS = cS+1;
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
        }
    }
    translate([0, lH, 0]) cube([lS, clipR, clipW]);
}

if (gen_top) translate([0, 0, clH]) case_top();
if (gen_bot) case_bottom();
if (gen_leg) case_leg();
