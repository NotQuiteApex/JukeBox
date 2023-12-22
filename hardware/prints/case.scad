$fn=12;

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
cpM = 9; // case pcb mounting plate size

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

ctH = 8; // case top height
ctW = 3; // case top wall
ctM = 8; // case top mounting plate size
ctMH = 3; // case top mountin plate height
cKBS = 17; // keyboard key size

kbX = cS / 2;   // keyboard key origin
kbY = 37 + ctW; // keyboard key origin

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
                    translate([ctW, ctW, -1]) roundedsquare(cS - ctW * 2, cS - ctW * 2, ctH+1, cR);
                }
            }
            translate([             0,              0, ctH-ctMH]) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);
            translate([cS - ctM - ctW,              0, ctH-ctMH]) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);
            translate([             0, cS - ctM - ctW, ctH-ctMH]) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);
            translate([cS - ctM - ctW, cS - ctM - ctW, ctH-ctMH]) roundedsquare(ctM+ctW, ctM+ctW, ctMH, cpR);
        }

        union() {
            // keyboard key holes
            for (w=[0:4-1]) {
                for (h=[0:3-1]) {
                    translate([kbX-30+w*20, kbY-20+h*20, ctH+0.5]) cube([cKBS, cKBS, 2], center=true);
                }
            }

            // mounting hardware holes
            translate([     7,      7, ctH-1]) { cylinder(d=3, h=5, center=true); translate([0, 0, 1]) cylinder(d2=7, d1=2, h=2, center=true); }
            translate([cS - 7,      7, ctH-1]) { cylinder(d=3, h=5, center=true); translate([0, 0, 1]) cylinder(d2=7, d1=2, h=2, center=true); }
            translate([     7, cS - 7, ctH-1]) { cylinder(d=3, h=5, center=true); translate([0, 0, 1]) cylinder(d2=7, d1=2, h=2, center=true); }
            translate([cS - 7, cS - 7, ctH-1]) { cylinder(d=3, h=5, center=true); translate([0, 0, 1]) cylinder(d2=7, d1=2, h=2, center=true); }
            
            // Jukebox logo
            translate([cS/2, cS-15, ctH+1]) linear_extrude(height=1, center=true) scale([0.035, 0.035, 1]) import(file="../../assets/textlogo.svg", center=true);
            // translate([cS/2-40, cS-20, ctH+1]) speaker_icon();
            // translate([cS/2+40, cS-20, ctH+1]) speaker_icon();
            // translate([cS/2, 6, ctH+1]) linear_extrude(height=1, center=true) text("friendteam.biz", size=4, halign="center", valign="center", font="Cascadia Mono:style=Regular");
        }
    }
}

translate([0, 0, clH]) case_top();
case_bottom();
