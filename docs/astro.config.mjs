import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	site: 'https://nix-community.github.io',
	base: '/crate2nix',
	integrations: [
		starlight({
			title: 'crate2nix',
			social: {
				github: 'https://github.com/nix-community/crate2nix',
			},
			editLink: {
				baseUrl: 'https://github.com/nix-community/crate2nix/edit/master/docs/',
			},
			sidebar: [
				{
					label: 'Guides',
					autogenerate: { directory: 'guides' },
				},
				{
					label: 'Contributing',
					autogenerate: { directory: 'contributing' },
				},
				{
					label: 'Reference',
					autogenerate: { directory: 'reference' },
				},
			],
		}),
	],
});
