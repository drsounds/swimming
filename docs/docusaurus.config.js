import {themes as prismThemes} from 'prism-react-renderer';

const baseUrl = '/swimming/';

const config = {
  title: 'Swimming',
  tagline: 'Native desktop apps with Rust and React',
  url: 'https://drsounds.github.io',
  baseUrl,
  organizationName: 'drsounds',
  projectName: 'swimming',
  onBrokenLinks: 'throw',
  headTags: [
    {
      tagName: 'script',
      attributes: {},
      innerHTML: `document.head.insertAdjacentHTML('afterbegin', '<base href="' + location.origin + '${baseUrl}">');`,
    },
  ],
  future: {
    experimental_router: 'hash',
  },
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },
  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.js',
        },
        blog: false,
      },
    ],
  ],
  themeConfig: {
    navbar: {
      title: 'Swimming',
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'tutorialSidebar',
          position: 'left',
          label: 'Documentation',
        },
        {
          href: 'https://github.com/drsounds/swimming',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  },
};

export default config;
