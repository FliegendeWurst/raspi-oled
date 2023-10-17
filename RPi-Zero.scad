module aligned_cube( size, align = [ 0, 0, 1 ] ) {
translate(size/2*[[align[0],0,0],[0,align[1],0],[0,0,align[2]]]) cube( size, center = true );
}
module aligned_cylinder( size, height, align = [ 0, 0, 1 ] ) {
translate([size,size,height]/2*[[align[0],0,0],[0,align[1],0],[0,0,align[2]]]) cylinder(height, size, size, center = true, $fn=32);
}

final = !$preview;
fn = 32;
module pizero() {
difference() {
color("green") import("pizero-scase_new.stl");
if (final) {
    translate([34.4, -25, 1.05]) cube([1,50,.5]);
    translate([34.5, -25, 1.15]) cube([1,50,.5]);
    translate([34.6, -25, 1.25]) cube([1,50,.5]);
    translate([34.7, -25, 1.35]) cube([1,50,.5]);
    translate([34.8, -25, 1.45]) cube([1,50,.5]);
    // keep 4.75 - 5.45 = 0.7
    h = 10.5;
    translate([34.8, -25, 5.45]) cube([1,50,h]);
    translate([34.7, -25, 5.55]) cube([1,50,h]);
    translate([34.6, -25, 5.65]) cube([1,50,h]);
    translate([34.5, -25, 5.75]) cube([1,50,h]);
    translate([34.4, -25, 5.85]) cube([1,50,h]);
}
translate([0,11.5,0]) cube([26*2, 6, 10], center=true);
}
}
module pizero_wall() {
projection(cut = true)
 translate([0,0,32]) rotate([0,90,0]) pizero();
 }
module oled_hole() {
intersection () {
    //translate([0,0,0]) import("OLED_Under_Cabinet.stl");
    translate([0,0,0]) union() {
    x = 19.75;
    y = 16;
        translate([-x, 0, -y]) rotate([90,0,0])
            cylinder(10, 1.1, 1.1, center=true, $fn=fn);
        translate([x, 0, -y]) rotate([90,0,0])
            cylinder(x, 1.1, 1.1, center=true, $fn=fn);
        translate([-x, 0, y]) rotate([90,0,0])
            cylinder(x, 1.1, 1.1, center=true, $fn=fn);
        translate([x, 0, y]) rotate([90,0,0])
            cylinder(10, 1.1, 1.1, center=true, $fn=fn);
        translate([0,0,3]) cube([15.87*2,10,14.5*2], center=true);
    };
    cube([48, 10, 40], center=true);
}
}
module led_hole() {
    translate([0, 19, 25]) {
        rotate([90, 0, 0]) cylinder(10, 5.7 / 2, 5.7 / 2, center = true, $fn=fn);
    }
}
module extender() {
    difference() {
    s = 90 - 8.5;
    //s = 12.5;
    h = s;
    union() {
        /*
        translate([0, 19, 60]) {
        color("red") rotate([90, 0, 0]) cylinder(10, 1, center = true);
    }
        */
        translate([0,0,8.5 + s/2]) linear_extrude(s, center=true) projection(cut = true) translate([0,0,-8]) pizero();
        translate([33, -15, 8.5]) cube([1,30,s]);
        difference() {
        union() {
            translate([0,0,8]) cube([34*2,33,2], center=true);
            translate([0,0,9]) cube([34*2,33,1], center=true);
            }
            union () {
                scale([0.99, 0.99, 1]) pizero();
                translate([-33,17.5,8.5]) cube([3.4,3.4,10], center=true);
                translate([-34.9,16,8.5]) cube([3.4,3.4,10], center=true);
                translate([33,17.5,8.5]) cube([3.4,3.4,10], center=true);
                translate([34.9,16,8.5]) cube([3.4,3.4,10], center=true);
                mirror([0,1,0]) {
                translate([-33,17.5,8.5]) cube([3.4,3.4,10], center=true);
                translate([-34.9,16,8.5]) cube([3.4,3.4,10], center=true);
                translate([33,17.5,8.5]) cube([3.4,3.4,10], center=true);
                translate([34.9,16,8.5]) cube([3.4,3.4,10], center=true);
                
                }
                translate([0,0,8.5]) cube([32*2,14.5*2,10],center=true);
                //translate([34,0,8.5]) cube([5,14.5*2,10],center=true);
            }
            }
    }
    translate([34.4, -25, 8.5]) cube([1,50,h]);
    /*
    translate([0, 19, 60]) difference() {
        cube([48,10,40], center=true);
        oled_hole();
    }
    */
    translate([0, 17, 60]) oled_hole();
    //led_hole();
    translate([-10,0,0])
        led_hole();
    translate([-20,0,0])
        led_hole();
    translate([10,0,0])
        led_hole();
    translate([20,0,0])
        led_hole();
    // cable hole
    translate([35,0,70]) {
        minkowski()
{
  color("red") cube([5,5,2], center=true);
  rotate([0,90,0]) cylinder(r1=2,r2=2,h=1, center=true, $fn=32);
}
    }
    // icons
    translate([0, 17.9 - .5, 41]) {
        scale = 0.19;
    translate([-14, 0, 0]) scale([scale, 1, scale]) rotate([90,0,0]) linear_extrude(1) import("./rpi_logo.svg", center=true);
    translate([-5, 0, 0]) scale([scale, 1, scale]) rotate([90,0,0]) linear_extrude(1) import("./Calendar_font_awesome.svg", center=true);
        translate([5, 0, 0]) scale([scale, 1, scale]) rotate([90,0,0]) linear_extrude(1) import("./to cry.svg", center=true);
        translate([14, 0, 0]) scale([scale, 1, scale]) mirror([1,0,0]) rotate([90,0,0]) linear_extrude(1) import("./temp.svg", center=true);
    }
}
}
module button_hole() {
    difference() {
        cube([12.4 + 0.05, 12.4 + 0.05,5], center=true);
        translate([6.05 + 0.05 / 2, -3.81, 0]) cube([0.3,2.6,5], center=true);
        translate([6.05 + 0.05 / 2, 3.81, 0]) cube([0.3,2.6,5], center=true);
        translate([-6.05 - 0.05 / 2, -3.81, 0]) cube([0.3,2.6,5], center=true);
        translate([-6.05 - 0.05 / 2, 3.81, 0]) cube([0.3,2.6,5], center=true);
    }
}
module hat() {
    difference() {
    translate([0,0,98.5]) mirror([0,0,1]) difference() {
        union() {
            pizero();
            color("red") translate([33, -13.5, 2.65]) cube([1.4,27,5.85]);
            color("red") translate([-33-1.4, -13.5, 2.65]) cube([1.4,27,5.85]);
            color("red") translate([-30, -18.3+1.4, 2.4]) cube([60,1.4,5.85]);
            translate([-30, -15, 0]) cube([60,30,1.4]);
            translate([0,0,8.5]) difference() {
                cube([66,32,4],center=true);
                cube([63,28,4],center=true);
            }
        }
        translate([34.4, -25, 1.05]) cube([1,50,5]);
        translate([0, 0, 0]) button_hole();
        translate([-20, 0, 0]) button_hole();
        translate([20, 0, 0]) button_hole();
    }
    scale([0.99, 0.99, 1]) extender();
}
}
module print1() {
translate([0,100,0])
difference() {
    intersection() {
        extender();
        translate([0,50,0]) cube([1000,100,1000], center=true);
    }
    translate([0,16,12]) aligned_cube([12.7,2,19]);
}
}
module print2() {
    intersection() {
        extender();
        mirror([0,1,0]) translate([0,50,0]) cube([1000,100,1000], center=true);
    }
}
module print3() {
difference() {
union() {
    hat();
    translate([0,0,97.8]) cube([55,16,1.4], center=true);
    }
    print9();
    translate([0,4,90]) aligned_cube([12,5,10]);
    }
}
module pyramid() {
difference() {
union() {
    for ( i = [0 : 8]) {
        s = (8 - i) / 2;
        translate([- s, - s, i * 0.5])
        cube([4+s,4+s,.5]);
        x = 2.5;
    }
    }
    x = 2.6;
        translate([4-x,4-x,1])
        cube([x,x,4]);
    }
}
module fixer() {
translate([0, 30, 1]) linear_extrude(2, center=true) circle(5, $fn=32);
    translate([-15, 25, 1]) rotate([0,0,20])scale([1,0.1,1]) linear_extrude(2, center=true) circle(20, $fn=fn);
    mirror([1,0,0]) translate([-15, 25, 1]) rotate([0,0,20])scale([1,0.1,1]) linear_extrude(2, center=true) circle(20, $fn=fn);
    
    translate([0, 50, 1]) linear_extrude(2, center=true) circle(5, $fn=fn);
    translate([0, 40, 1]) rotate([0,0,90])scale([.7,0.1,1]) linear_extrude(2, center=true) circle(20, $fn=fn);
    }
module print4() {
difference() {
union() {
    pizero();
    translate([-34.4 , -16.9, 0]) pyramid();
    translate([-34.4 , 116.9, 0]) rotate([0, 0, 270]) pyramid();
    translate([34.4 , -16.9, 0]) rotate([0,0,90]) pyramid();
    translate([34.4 , 116.9, 0]) rotate([0, 0, 180]) pyramid();
    /*
    fixer();
    mirror([0, 1, 0]) {
    fixer();
    }
    */
    translate([0,100,0]) difference() {
        pizero();
        translate([34.4,-15,0]) cube([5,100,10]);
    }
    }
    translate([34.4,-12,0]) cube([5,25,10]);
    translate([21.5, -17, 4.5]) cube([14, 4, 9], center=true);
    translate([21.5, -25, 4.5]) cube([14,20, 7], center=true);
    // remove divider
    translate([0,100,0]) translate([0, -17, 4.5]) cube([66, 5.5, 9], center=true);
    }

translate([-34.4 , 90, 8.5]) cube([1.4,2.8,2.8]);
translate([-34.4 , 10, 8.5]) cube([1.4,2.8,2.8]);
translate([33 , 90, 8.5]) cube([1.4,2.8,2.8]);
translate([33 , 10, 8.5]) cube([1.4,2.8,2.8]);
translate([0,15,0]) aligned_cube([66,100,1.4], [0,1,1]);
    translate([-17.57, 13, 0]) rotate([270, -90, 0]) intersection() {
        linear_extrude(100) pizero_wall();
        translate([-100,-110,0])
        cube([109,100,110]);
    }
    translate([17.57, 111, 0]) rotate([90, -90, 0]) intersection() {
        linear_extrude(100) pizero_wall();
        translate([-100,-110,0])
        cube([109,100,110]);
    }
}
module print5() {
difference() {
union() {
translate([-34.4, 16.91, 90]) cube([34.4*2,66.18,1]);
translate([-34.4, 14, 90]) cube([5,72,1]);
translate([29.4, 14, 90]) cube([5,72,1]);
mi = 8;
for (i = [0 : mi]) {
    translate([0, 30, 90-i-.5]) cube([22+(mi-i)/2,13+(mi-i)/2,1], center=true);
}
}
translate([0, 30, 90]) cube([20,11,20], center=true);
print8();
print3();
}
translate([8, 30, 89.5]) cube([4,5.5,3], center=true);
translate([-8, 30, 89.5]) cube([4,5.5,3], center=true);
translate([9.5, 30, 84]) cube([1,4,3], center=true);
translate([-9.5, 30, 84]) cube([1,4,3], center=true);

translate([-34.4 , 70, 87.2]) cube([1.4,2.8,2.8]);
translate([-34.4 , 30, 87.2]) cube([1.4,2.8,2.8]);
translate([33 , 70, 87.2]) cube([1.4,2.8,2.8]);
translate([33 , 30, 87.2]) cube([1.4,2.8,2.8]);
}
module print6() {
difference() {
    translate([-34.4, 0, 8.5]) cube([1.4,100,81.5]);
    print11();
    print12();
    print4();
    print5();
    translate([-34.4,36-23,76]) rotate([90,90,90]) aligned_cylinder(3,1.4);
}
}
module print7() {
difference() {
    translate([33, 0, 8.5]) cube([1.4,100,81.5]);
    print10();
    print4();
    print5();
    translate([33,27-15,72]) rotate([90,90,90]) aligned_cylinder(3,1.4);
}
}
module print8() {
translate([0,100,0])
    hat();
}
module holes() {
translate([-43/2,1,5]) rotate([90,0,0]) aligned_cylinder(1,1.6,[0,0,1]);
translate([43/2,1,5]) rotate([90,0,0]) aligned_cylinder(1,1.6,[0,0,1]);
translate([-43/2,1,5+17]) rotate([90,0,0]) aligned_cylinder(1,1.6,[0,0,1]);
translate([43/2,1,5+17]) rotate([90,0,0]) aligned_cylinder(1,1.6,[0,0,1]);
}
module print9() {
difference() {
union() {
    translate([0,0,100]) rotate([22.5, 0, 0])
    difference() {
        // real size: 45mm x 20mm
        minkowski() {
            translate([0,0.1,0]) aligned_cube([45,1.2,25], [0,0,1]);
            rotate([90,0,0]) aligned_cylinder(1,0.1);
        }
        holes();
    };
    difference() {
        translate([0,0,96]) aligned_cube([45,1.4,5], [0,0,1]);
        translate([0,0,100]) rotate([22.5, 0, 0]) holes();
    }
}
translate([0,0,100]) rotate([22.5, 0, 0])
translate([0,1.4,-1]) aligned_cube([45,1.4,21], [0,0,1]);
}
}
module all() {
    print1();
    print2();
    print3();
    print4();
    print5();
    print6();
    print7();
    print8();
    print9();
    print10();
    print11();
    print12();
}
module print10() {
    translate([45.4,40,70]) difference() {
        union() {
        minkowski() {
            translate([0,0,0.1]) aligned_cube([20,20,1.2]);
            aligned_cylinder(1,0.2,[0,0,0]);
        }
        translate([-11-1.4, 5,0]) aligned_cube([1.4,2.8,1.4], [1,0,1]);
        translate([-11-1.4, -5,0]) aligned_cube([1.4,2.8,1.4], [1,0,1]);
        }
        translate([7.5,-7.5,.7]) aligned_cylinder(1.5,1.4,[0,0,0]);
        translate([-7.5,-7.5,.7]) aligned_cylinder(1.5,1.4,[0,0,0]);
        translate([0,-7.5,0]) aligned_cube([10,4,1.4]);
        translate([0,1.5,0]) aligned_cube([10,4,1.4]);
    }
}
module print11() {

translate([-48.4,40,70]) cube([10,24,1.4]);
translate([-40.4+2,45,70]) cube([1.4+4,2.8,1.4]);
translate([-40.4+2,55,70]) cube([1.4+4,2.8,1.4]);
translate([-40.4+4+.6,55,70]) cube([1.4,2.8+.8,1.4]);
translate([-40.4+4+.6,55-1,70]) cube([1.4,2.8+.8,1.4]);
translate([-40.4+4+.6,45,70]) cube([1.4,2.8+.8,1.4]);
translate([-40.4+4+.6,45-1,70]) cube([1.4,2.8+.8,1.4]);
}
module print12() {
translate([0,4,0]) {
translate([-48.4,40,76]) cube([10,20,1.4]);
translate([0,-2, 0]) {
translate([-40.4+2,45,76]) cube([1.4+4,2.8,1.4]);
translate([-40.4+2,55,76]) cube([1.4+4,2.8,1.4]);
translate([-40.4+4+.6,55,76]) cube([1.4,2.8+.8,1.4]);
translate([-40.4+4+.6,55-1,76]) cube([1.4,2.8+.8,1.4]);
translate([-40.4+4+.6,45,76]) cube([1.4,2.8+.8,1.4]);
translate([-40.4+4+.6,45-1,76]) cube([1.4,2.8+.8,1.4]);
}
}
}
print5();
print7();
print6();