import React, {ReactElement} from 'react';
import {Form, FormProps} from 'ink-form';
import {useFullscreen} from '../utils/usefullscreen.js';

type Props = {
	form: FormProps;
	callback: (result: object) => void;
};

export default function PostDashboard({form, callback}: Props): ReactElement {
	useFullscreen();

	return <Form {...form} onSubmit={result => callback(result)} />;
}
