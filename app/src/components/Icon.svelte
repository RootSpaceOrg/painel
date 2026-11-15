<script lang="ts">
  // Traço fino estilo lucide, herda cor do texto.
  const P: Record<string, string> = {
    play: 'M7 4.5v15l12-7.5z',
    pause: 'M7 4h3.5v16H7zM13.5 4H17v16h-3.5z',
    next: 'M5 4.5v15l10-7.5zM16 4.5h2.5v15H16z',
    prev: 'M19 4.5v15L9 12zM5.5 4.5H8v15H5.5z',
    check: 'M4.5 12.5l5 5L19.5 7',
    x: 'M6 6l12 12M18 6L6 18',
    plus: 'M12 5v14M5 12h14',
    edit: 'M4 20h4L19 9l-4-4L4 16zM13.5 6.5l4 4',
    expand: 'M14 4h6v6M10 20H4v-6M20 4l-7 7M4 20l7-7',
    gear: 'M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7zM19.4 13.5l1.6 1.2-2 3.4-1.9-.7a7.6 7.6 0 0 1-2.2 1.3L14.6 21h-4l-.3-2.3a7.6 7.6 0 0 1-2.2-1.3l-1.9.7-2-3.4 1.6-1.2a7.7 7.7 0 0 1 0-3L4.2 9.3l2-3.4 1.9.7a7.6 7.6 0 0 1 2.2-1.3L10.6 3h4l.3 2.3a7.6 7.6 0 0 1 2.2 1.3l1.9-.7 2 3.4-1.6 1.2a7.7 7.7 0 0 1 0 3z',
    volume: 'M4 9.5h3.5L12 5v14l-4.5-4.5H4zM16 9a4 4 0 0 1 0 6M18.5 6.5a7.5 7.5 0 0 1 0 11',
    mute: 'M4 9.5h3.5L12 5v14l-4.5-4.5H4zM16 9.5l5 5M21 9.5l-5 5',
    trash: 'M5 7h14M10 7V4.5h4V7M7 7l1 13h8l1-13',
    heart: 'M12 20s-7.5-4.6-7.5-10A4.3 4.3 0 0 1 12 7.6 4.3 4.3 0 0 1 19.5 10c0 5.4-7.5 10-7.5 10z',
    refresh: 'M20 11a8 8 0 0 0-14.6-4.5M4 4v4h4M4 13a8 8 0 0 0 14.6 4.5M20 20v-4h-4',
    music: 'M9 18V5l11-2v13M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0zM20 16a3 3 0 1 1-6 0 3 3 0 0 1 6 0z',
    flame: 'M12 21c-3.9 0-6.5-2.6-6.5-6.2 0-3.7 3.3-5.6 3.8-9.8 2.6 1.4 4 3.6 4 5.6 1-.6 1.6-1.6 1.8-2.8 2.1 1.6 3.4 4 3.4 7 0 3.6-2.6 6.2-6.5 6.2z',
    calendar: 'M4 6.5h16V20H4zM4 10.5h16M8.5 3.5v5M15.5 3.5v5',
    speaker: 'M6 3.5h12v17H6zM12 15.5a2.5 2.5 0 1 0 0-.01M12 7.5v.01',
    list: 'M9 6h11M9 12h11M9 18h11M4.5 6h.01M4.5 12h.01M4.5 18h.01',
    shuffle: 'M3 7h3.5l9 10H21M3 17h3.5l2.8-3.1M13.7 10.1L16.5 7H21M18 4l3 3-3 3M18 14l3 3-3 3',
    repeat: 'M4 11V9.5A2.5 2.5 0 0 1 6.5 7H20l-3-3M20 13v1.5a2.5 2.5 0 0 1-2.5 2.5H4l3 3',
    lyrics: 'M4 6h16M4 10.5h11M4 15h16M4 19.5h8',
    down: 'M6 9l6 6 6-6',
    image: 'M4 5h16v14H4zM4 16l5-5 4 4 2.5-2.5L20 17M15.5 9.5h.01',
  };
  let { name, filled = false }: { name: string; filled?: boolean } = $props();
  const fill = $derived(filled || name === 'play' || name === 'pause' || name === 'next' || name === 'prev');
</script>

<svg viewBox="0 0 24 24" aria-hidden="true" fill={fill ? 'currentColor' : 'none'} stroke="currentColor"
  stroke-width={fill ? 0 : 1.7} stroke-linecap="round" stroke-linejoin="round">
  <path d={P[name]} />
</svg>
