declare module 'ink-image' {
	import {FC} from 'react';

	interface ImageProps {
		src: string;
		preserveAspectRatio?: boolean;
		width?: number;
		height?: number;
		alt?: string;
	}

	const Image: FC<ImageProps>;
	export default Image;
}
