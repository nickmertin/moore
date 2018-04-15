-- @elab pkg_a
package pkg_a is
end package;

-- @ elab pkg_b
package pkg_b is
	type BYTE is range 0 to 255;
	constant K : BYTE;
end package;

-- @ elab pkg_c
package pkg_c is
	use work.pkg_b.BYTE;
	type SHORT is range 0 to 65535;
	type INT is range 0 to 4294967295;
	constant K0 : BYTE;
	constant K1 : SHORT;
	constant K2 : INT;
end package;

package pkg_d is
	use work.pkg_b.BYTE;
	use work.pkg_c.all;
	constant K0 : SHORT;
	constant K1 : INT;
end package;
