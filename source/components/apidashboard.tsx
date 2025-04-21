import React, {useEffect, useState} from 'react';
import PostDashboard from './postdashboard.js';
import GetDashboard from './getdashboard.js';
import {fetchApi} from '../utils/fetchapi.js';

type Props = {
	method: string;
	endpoint: string;
	resultObject: string; // Where the results of the API call are stored on the object
};

export default function ApiDashboard({method, endpoint, resultObject}: Props) {
	const [loading, data] = fetchApi({method, endpoint, resultObject});
	if (method === 'POST') {
		return <PostDashboard form={data} callback={result => callback(result)} />;
	} else if (method === 'GET') {
		return <GetDashboard form={data} callback={result => callback(result)} />;
	}
}
