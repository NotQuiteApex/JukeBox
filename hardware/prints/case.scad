$fn=32;

// https://www.youtube.com/watch?v=gKOkJWiTgAY
module roundedsquare(xdim, ydim, zdim, rdim){
    hull() {
        translate([       rdim,        rdim, 0]) cylinder(h=zdim, r=rdim);
        translate([xdim - rdim,        rdim, 0]) cylinder(h=zdim, r=rdim);
        translate([       rdim, ydim - rdim, 0]) cylinder(h=zdim, r=rdim);
        translate([xdim - rdim, ydim - rdim, 0]) cylinder(h=zdim, r=rdim);
    }
}
module chamferedsquare(xdim, ydim, zdim, r1dim, r2dim){
    bigrdim = max(r1dim, r2dim);
    hull() {
        translate([       bigrdim,        bigrdim, 0]) cylinder(h=zdim, r1=r1dim, r2=r2dim);
        translate([xdim - bigrdim,        bigrdim, 0]) cylinder(h=zdim, r1=r1dim, r2=r2dim);
        translate([       bigrdim, ydim - bigrdim, 0]) cylinder(h=zdim, r1=r1dim, r2=r2dim);
        translate([xdim - bigrdim, ydim - bigrdim, 0]) cylinder(h=zdim, r1=r1dim, r2=r2dim);
    }
}

cS = 98; // case size
cR = 3;  // case radius (rounded corners)

clH = 3; // case lip height
clC = 1; // case lip chamfer
clS = 3; // case lip size

cpR = 3; // case pcb radius
cpH = 3; // case pcb height
cpW = 3; // case pcb wall
cpM = 9; // case pcb mounting pole size

module case_bottom() {
    difference() {
        union() {
            // Case floor
            chamferedsquare(cS, cS, clC, cR - clC, cR);
            translate([0, 0, clC]) roundedsquare(cS, cS, clH - clC, cR);

            // PCB table
            difference() {
                translate([      clS,       clS, clH]) roundedsquare(cS - clS * 2, cS - clS * 2, cpH, cpR);
                translate([clS + cpW, clS + cpW, clH]) cube([cS - clS * 2 - cpW * 2, cS - clS * 2 - cpW * 2, cpH]);
            }
            translate([                   clS,                    clS, clH]) roundedsquare(cpM, cpM, cpH, cpR);
            translate([cS - clS * 2 - cpW * 2,                    clS, clH]) roundedsquare(cpM, cpM, cpH, cpR);
            translate([                   clS, cS - clS * 2 - cpW * 2, clH]) roundedsquare(cpM, cpM, cpH, cpR);
            translate([cS - clS * 2 - cpW * 2, cS - clS * 2 - cpW * 2, clH]) roundedsquare(cpM, cpM, cpH, cpR);

            // USB-C pillar
            translate([0, 70.5, clH]) cube([clS, 10.5, cpW+1.6]);
        }

        union() {
            // Bolt holes
            translate([     7,      7, 6]) cylinder(d=3, h=14, center=true);
            translate([cS - 7,      7, 6]) cylinder(d=3, h=14, center=true);
            translate([     7, cS - 7, 6]) cylinder(d=3, h=14, center=true);
            translate([cS - 7, cS - 7, 6]) cylinder(d=3, h=14, center=true);

            // Nut holes
            translate([     7,      7, 1]) cylinder($fn=6, r=3, h=2, center=true);
            translate([cS - 7,      7, 1]) cylinder($fn=6, r=3, h=2, center=true);
            translate([     7, cS - 7, 1]) cylinder($fn=6, r=3, h=2, center=true);
            translate([cS - 7, cS - 7, 1]) cylinder($fn=6, r=3, h=2, center=true);

            // Hole for screen cable
            translate([cS/2, cS-clS-cpW/2, clH+cpH/2]) cube([24, cpW, cpH], center=true);
        }
    }
}

ctH = 7; // case top height
ctW = 3; // case top wall

module case_top() {
    translate([0, 0, ctH]) chamferedsquare(cS, cS, 1, 3, 2);
    difference() {
        roundedsquare(cS, cS, ctH, cR);
        translate([ctW, ctW, 0]) roundedsquare(cS - ctW * 2, cS - ctW * 2, ctH, cR);
    }
}

// case_top();
case_bottom();
