import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { TanStackRouterVite } from '@tanstack/router-plugin/vite'
import fs from 'fs'
import path from 'path'
import { execFileSync } from 'child_process'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

const getMarkdownMetadata = () => {
  const staticDir = path.resolve(__dirname, 'static')
  if (!fs.existsSync(staticDir)) return {}

  const files = fs.readdirSync(staticDir).filter(file => file.endsWith('.md'))
  const metadata: Record<string, { title: string; lastUpdated: string }> = {}

  files.forEach(file => {
    const filePath = path.join(staticDir, file)
    let lastUpdated = ''
    try {
      // Use git log to get the last commit date
      lastUpdated = execFileSync('git', ['log', '-1', '--format=%cd', '--date=iso-strict', filePath]).toString().trim()
    } catch (e) {
      console.warn(`Could not get git timestamp for ${file}`, e)
    }

    // fallback if git log returns empty (e.g. not committed yet)
    if (!lastUpdated) {
        try {
            const stats = fs.statSync(filePath)
            lastUpdated = stats.mtime.toISOString()
        } catch {
            lastUpdated = new Date().toISOString()
        }
    }

    // Read file content to parse frontmatter if available
    let title = ''
    try {
        const content = fs.readFileSync(filePath, 'utf-8')
        // Simple regex to find title = "..." in TOML frontmatter
        const frontmatterMatch = content.match(/^\+\+\+([\s\S]*?)\+\+\+/)
        if (frontmatterMatch) {
            const frontmatter = frontmatterMatch[1]
            const titleMatch = frontmatter.match(/title\s*=\s*"(.*?)"/)
            if (titleMatch) title = titleMatch[1]
        }
    } catch (e) {
        console.warn(`Could not parse frontmatter for ${file}`, e)
    }

    const name = path.basename(file, '.md')

    // Fallback to filename if no title in frontmatter
    if (!title) {
        // Convert kebab-case to Title Case
        title = name.split('-').map(word => word.charAt(0).toUpperCase() + word.slice(1)).join(' ')
    }

    metadata[name] = {
      title,
      lastUpdated
    }
  })
  return metadata
}

const markdownMetadata = getMarkdownMetadata()


// https://vite.dev/config/
export default defineConfig({
  define: {
    __MARKDOWN_METADATA__: JSON.stringify(markdownMetadata)
  },
  plugins: [
    TanStackRouterVite(),
    react()
  ],
  envDir: '../../',
  server: {
    host: true,
    watch: {
        usePolling: true
    }
  },
  preview: {
    host: true,
    port: 4173
  },
  build: {
    rollupOptions: {
      output: {
      manualChunks: (id) => {
        if (id.includes('node_modules')) {
          if (id.includes('@mysten')) return 'mysten';
          if (id.includes('@tanstack')) return 'tanstack';
          if (id.includes('lucide-react')) return 'lucide';
          if (id.includes('zod')) return 'zod';
          return 'vendor';
        }
      }
      }
    }
  }
})
