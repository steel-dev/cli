export async function uploadProfileToSteel(
	zipBuffer: Buffer,
	apiKey: string,
	apiBase: string,
): Promise<string> {
	const form = new FormData();
	form.append(
		'userDataDir',
		new Blob([new Uint8Array(zipBuffer)], {type: 'application/zip'}),
		'userDataDir.zip',
	);

	const res = await fetch(`${apiBase}/profiles`, {
		method: 'POST',
		headers: {'Steel-Api-Key': apiKey},
		body: form,
	});

	const body = (await res.json()) as {id?: string; message?: string};

	if (!res.ok) {
		throw new Error(
			`Profile upload failed (${res.status}): ${body.message ?? JSON.stringify(body)}`,
		);
	}

	if (!body.id) {
		throw new Error('Profile upload response missing id');
	}

	return body.id;
}

export async function updateProfileOnSteel(
	profileId: string,
	zipBuffer: Buffer,
	apiKey: string,
	apiBase: string,
): Promise<void> {
	const form = new FormData();
	form.append(
		'userDataDir',
		new Blob([new Uint8Array(zipBuffer)], {type: 'application/zip'}),
		'userDataDir.zip',
	);

	const res = await fetch(`${apiBase}/profiles/${profileId}`, {
		method: 'PATCH',
		headers: {'Steel-Api-Key': apiKey},
		body: form,
	});

	if (!res.ok) {
		const body = (await res.json()) as {message?: string};
		throw new Error(
			`Profile update failed (${res.status}): ${body.message ?? JSON.stringify(body)}`,
		);
	}
}
