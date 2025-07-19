import React from 'react';
import Success from '../success.js';
import {useTask} from '../../hooks/usetask.js';
import {useForgeStep} from '../../context/forgestepcontext.js';

export default function ForgeSuccess() {
	const {step, setStep} = useForgeStep();
	const [state, task, , , setTask] = useTask();

	return step === 'success' ? <Success /> : null;
}
