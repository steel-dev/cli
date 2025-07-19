#!/usr/bin/env node

import React, {useState, useEffect} from 'react';
import {Box, Text} from 'ink';

// The static ASCII logo that will remain fixed
const logoLines = [
	' @@@@@@@@@@@@@@@@@@@@@@@@@@@@@ ',
	'@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@',
	'@@@@@@@@@9999999999999@@@@@@@@@',
	'@@@@@[                   ]@@@@@',
	'@@@@[                      @@@@',
	'@@@@     @@@@@@@@@@@@@     @@@@',
	'@@@@B              @@@     @@@@',
	'@@@@@@               @     @@@@',
	'@@@@@@@@@@@@@@@@     @     @@@@',
	'@@@@                 @     @@@@',
	'@@@@                @@     @@@@',
	'@@@@@@@@@@@@@@@g@@@@@@@@@@@@@@@',
	'@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@',
	' @@@@@@@@@@@@@@@@@@@@@@@@@@@@@ ',
];

//
// Utility function to convert HSL to HEX
// h: 0 to 360, s: 0 to 100, l: 0 to 100
//
function hslToHex(h: number, s: number, l: number): string {
	s /= 100;
	l /= 100;
	const c = (1 - Math.abs(2 * l - 1)) * s;
	const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
	const m = l - c / 2;
	let r = 0,
		g = 0,
		b = 0;

	if (h >= 0 && h < 60) {
		r = c;
		g = x;
		b = 0;
	} else if (h >= 60 && h < 120) {
		r = x;
		g = c;
		b = 0;
	} else if (h >= 120 && h < 180) {
		r = 0;
		g = c;
		b = x;
	} else if (h >= 180 && h < 240) {
		r = 0;
		g = x;
		b = c;
	} else if (h >= 240 && h < 300) {
		r = x;
		g = 0;
		b = c;
	} else {
		r = c;
		g = 0;
		b = x;
	}

	const toHex = (n: number) => {
		const hex = Math.round((n + m) * 255).toString(16);
		return hex.length === 1 ? '0' + hex : hex;
	};

	return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
}

// Compute wavy color based on row, col, and an offset.
// This creates a non-linear pattern by combining sine and cosine.
function getWavyColor(row: number, col: number, offset: number): string {
	// Combine sine and cosine to generate a wave-like effect
	const wave = Math.sin((col + offset) / 5) + Math.cos((row + offset) / 3);
	// wave is now in the range of approximately [-2, 2]. Normalize it to [0, 360].
	const normalized = ((wave + 2) / 4) * 360;
	// Use the normalized value as the hue.
	const hue = normalized % 360;
	return hslToHex(hue, 100, 50);
}

export default function AnimatedLogo() {
	// This offset state drives the animation.
	const [offset, setOffset] = useState(0);

	useEffect(() => {
		// Update the offset every second (adjust speed here if needed).
		const interval = setInterval(() => {
			setOffset(prev => (prev + 5) % 360);
		}, 400);
		return () => clearInterval(interval);
	}, []);

	// Render the logo with a wavy color pattern.
	const renderAnimatedLogo = () =>
		logoLines.map((line, rowIndex) => {
			const renderedRow = [...line].map((char, colIndex) => {
				// Here we use the wavy color helper.
				const color = getWavyColor(rowIndex, colIndex, offset);
				return (
					<Text key={colIndex} color={color}>
						{char}
					</Text>
				);
			});
			return <Box key={rowIndex}>{renderedRow}</Box>;
		});

	return renderAnimatedLogo();
}
