<script>
  import { imageToken } from '../../lib/theme.js'

  export let path = ''

  const IMAGE_RE = /\.(png|jpg|jpeg|gif|webp|svg|avif|bmp|ico)$/i
  const VIDEO_RE = /\.(mp4|webm|ogg|mov|avi)$/i
  const AUDIO_RE = /\.(mp3|wav|ogg|m4a|flac)$/i

  $: type = IMAGE_RE.test(path) ? 'image'
          : VIDEO_RE.test(path) ? 'video'
          : AUDIO_RE.test(path) ? 'audio'
          : 'file'
  $: filename = path.split('/').pop()
  $: assetUrl = path && $imageToken ? `/api/vault-asset/${path.split('/').map(encodeURIComponent).join('/')}?token=${$imageToken}` : ''
</script>

<div class="asset-viewer">
  {#if type === 'image'}
    <div class="image-wrap">
      <img src={assetUrl} alt={filename} />
    </div>
  {:else if type === 'video'}
    <div class="media-wrap">
      <!-- svelte-ignore a11y-media-has-caption -->
      <video controls src={assetUrl}></video>
    </div>
  {:else if type === 'audio'}
    <div class="media-wrap">
      <audio controls src={assetUrl}></audio>
    </div>
  {:else}
    <div class="file-info">
      <svg width="48" height="48" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" class="file-icon">
        <path d="M13.5 7.5l-6 6a4 4 0 01-5.657-5.657l6.364-6.364a2.5 2.5 0 013.536 3.536L5.379 11.35a1 1 0 01-1.415-1.414l5.657-5.657"/>
      </svg>
      <p class="filename">{filename}</p>
      {#if assetUrl}
        <a href={assetUrl} download={filename} class="download-btn">Download</a>
      {/if}
    </div>
  {/if}
</div>

<style>
  .asset-viewer {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 2rem;
    overflow: auto;
  }

  .image-wrap {
    max-width: 100%;
    max-height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .image-wrap img {
    max-width: 100%;
    max-height: calc(100vh - 120px);
    object-fit: contain;
    border-radius: 6px;
    box-shadow: 0 4px 24px rgba(0,0,0,0.3);
  }

  .media-wrap {
    width: 100%;
    max-width: 800px;
  }

  .media-wrap video {
    width: 100%;
    border-radius: 6px;
  }

  .media-wrap audio {
    width: 100%;
  }

  .file-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    color: var(--color-text-muted);
  }

  .file-icon {
    opacity: 0.4;
  }

  .filename {
    font-size: 1rem;
    font-weight: 500;
    color: var(--color-text);
    word-break: break-all;
    text-align: center;
  }

  .download-btn {
    display: inline-flex;
    align-items: center;
    padding: 0.4rem 1.2rem;
    background: var(--color-accent);
    color: #000;
    font-weight: 600;
    font-size: 0.85rem;
    border-radius: 6px;
    text-decoration: none;
    transition: opacity 0.1s;
  }

  .download-btn:hover {
    opacity: 0.85;
  }
</style>
