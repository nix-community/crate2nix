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
					autogenerate: { directory: '00_guides' },
				},
				{
					label: 'Contributing',
					autogenerate: { directory: '50_contributing' },
				},
				{
					label: 'Design & Background',
					autogenerate: { directory: '70_design' },
				},
				{
					label: 'Reference',
					autogenerate: { directory: '90_reference' },
				},
			],
		}),
	],
});
