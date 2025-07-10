import {ReactElement} from 'react';
import {Form, FormProps} from 'ink-form';
// import {useFullscreen} from '../hooks/usefullscreen.js';

type Props = {
	form: FormProps;
	callback: (result: Record<string, any>) => void;
};

export default function PostDashboard({form, callback}: Props): ReactElement {
	// useFullscreen();

	return <Form {...form} onSubmit={result => callback(result)} />;
}
