// JukeBox Case

/* [General settings] */
// Generates top case piece
gen_top = false;
// Generates bottom case piece
gen_bot = false;
// Generates screen case piece
gen_scr = false;
// Case size (width & height)
cS = 98;
// Case corner radius (rounded corners)
cR = 3;
// Face count on rounded objects
$fn=12;

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

/* [Case top settings] */
// Case top height
ctH = 8;
// Case top wall size
ctW = 3;
// Case top mounting plate size
ctM = 8;
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
kbY = 40;
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
            translate([   7,    7, 6]) cylinder(d=3, h=14, center=true);
            translate([cS-7,    7, 6]) cylinder(d=3, h=14, center=true);
            translate([   7, cS-7, 6]) cylinder(d=3, h=14, center=true);
            translate([cS-7, cS-7, 6]) cylinder(d=3, h=14, center=true);

            // Nut holes
            translate([   7,    7, 1]) cylinder($fn=6, r=3, h=2, center=true);
            translate([cS-7,    7, 1]) cylinder($fn=6, r=3, h=2, center=true);
            translate([   7, cS-7, 1]) cylinder($fn=6, r=3, h=2, center=true);
            translate([cS-7, cS-7, 1]) cylinder($fn=6, r=3, h=2, center=true);

            // Hole for screen cable
            translate([cS/2, cS-clS-cpW/2, clH+cpH/2]) cube([24, cpW, cpH], center=true);
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


module case_top() {
    difference() {
        union() {
            difference() {
                union() {
                    // top shell
                    translate([0, 0, ctH]) chamferedsquare(cS, cS, 1, 3, 2);
                    roundedsquare(cS, cS, ctH, cR);
                }
                union() {
                    // USB-C hole
                    translate([-1, 70.5, -1]) cube([ctW+2, 10.5, ctH+1]);
                    // Interior
                    translate([ctW, ctW, -1]) roundedsquare(cS-ctW*2, cS-ctW*2, ctH+1, cR);
                }
            }
            // Mounting plates
            translate([         0,          0, ctH-ctMH]) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);
            translate([cS-ctM-ctW,          0, ctH-ctMH]) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);
            translate([         0, cS-ctM-ctW, ctH-ctMH]) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);
            translate([cS-ctM-ctW, cS-ctM-ctW, ctH-ctMH]) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);
        }

        union() {
            // keyboard key holes
            for (w=[0:kbW-1]) {
                for (h=[0:kbH-1]) {
                    translate([kbX-kbOX+w*kbSW, kbY-kbOY+h*kbSH, ctH+1]) cube([kbS, kbS, 2], center=true);
                }
            }

            // mounting hardware holes
            translate([   7,    7, ctH-1]) { cylinder(d=3, h=5, center=true); translate([0, 0, 1]) cylinder(d2=7, d1=2, h=2, center=true); }
            translate([cS-7,    7, ctH-1]) { cylinder(d=3, h=5, center=true); translate([0, 0, 1]) cylinder(d2=7, d1=2, h=2, center=true); }
            translate([   7, cS-7, ctH-1]) { cylinder(d=3, h=5, center=true); translate([0, 0, 1]) cylinder(d2=7, d1=2, h=2, center=true); }
            translate([cS-7, cS-7, ctH-1]) { cylinder(d=3, h=5, center=true); translate([0, 0, 1]) cylinder(d2=7, d1=2, h=2, center=true); }
            
            // Jukebox logo
            translate([logoX, logoY, ctH+1]) linear_extrude(height=1, center=true) scale([logoS, logoS, 1]) import(file="../assets/textlogo.svg", center=true);
            // translate([cS/2-40, cS-20, ctH+1]) speaker_icon();
            // translate([cS/2+40, cS-20, ctH+1]) speaker_icon();
            // translate([cS/2, 6, ctH+1]) linear_extrude(height=1, center=true) text("friendteam.biz", size=4, halign="center", valign="center", font="Cascadia Mono:style=Regular");
        }
    }
}

if (gen_top) translate([0, 0, clH]) case_top();
if (gen_bot) case_bottom();
