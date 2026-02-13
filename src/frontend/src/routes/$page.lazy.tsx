import { createLazyFileRoute } from '@tanstack/react-router'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { useEffect, useState } from 'react'
import { DashboardLayout } from '../components/DashboardLayout'

export const Route = createLazyFileRoute('/$page')({
  component: MarkdownPage,
})

interface PageMetadata {
    title: string
    author?: string
    effectiveDate?: string
    lastUpdated: string
}

function parseFrontmatter(raw: string): { content: string; frontmatter: Record<string, string> } {
    const match = raw.match(/^\+\+\+([\s\S]*?)\+\+\+/)
    if (!match) return { content: raw, frontmatter: {} }

    const frontmatterRaw = match[1]
    const content = raw.slice(match[0].length).trim()
    const frontmatter: Record<string, string> = {}

    frontmatterRaw.split('\n').forEach(line => {
        const parts = line.split('=')
        if (parts.length >= 2) {
            const key = parts[0].trim()
            let value = parts.slice(1).join('=').trim()
            if (value.startsWith('"') && value.endsWith('"')) {
                value = value.slice(1, -1)
            }
            frontmatter[key] = value
        }
    })

    return { content, frontmatter }
}

function MarkdownPage() {
  const { metadata: buildMetadata, page } = Route.useLoaderData()
  const [content, setContent] = useState<string>('')
  const [pageMetadata, setPageMetadata] = useState<PageMetadata>({
      title: buildMetadata.title,
      lastUpdated: buildMetadata.lastUpdated
  })

  useEffect(() => {
    const loadContent = async () => {
      try {
        const modules = import.meta.glob('../../static/*.md', { query: '?raw', import: 'default' })
        const path = `../../static/${page}.md`

        if (modules[path]) {
          const rawContent = await modules[path]() as string
          const { content: cleanContent, frontmatter } = parseFrontmatter(rawContent)

          // Extract Effective Date from content if not in frontmatter
          let effectiveDate = frontmatter['date'] // Common key
          if (!effectiveDate) {
              const effectiveMatch = cleanContent.match(/\*\*Effective Date:\*\*\s+(\d{4}-\d{2}-\d{2})/)
              if (effectiveMatch) {
                  effectiveDate = effectiveMatch[1]
              }
          }

          setPageMetadata(prev => ({
              ...prev,
              title: frontmatter['title'] || prev.title,
              author: frontmatter['author'],
              effectiveDate,
              lastUpdated: prev.lastUpdated // Keep build time git timestamp
          }))

          // Strip metadata lines from content to avoid duplication
          // Removes lines like "**Effective Date:** 2023-01-01" and "**Last Updated:** {{ .lastUpdated }}"
          const injectedContent = cleanContent
            .replace(/\*\*Effective Date:\*\*.*\n?/g, '')
            .replace(/\*\*Last Updated:\*\*.*\n?/g, '')
            .trim()

          setContent(injectedContent)
        }
      } catch (e) {
        console.error('Failed to load markdown content', e)
      }
    }
    loadContent()
  }, [page, buildMetadata])

  const formatDate = (dateString?: string) => {
      if (!dateString) return 'N/A'
      try {
          return new Date(dateString).toISOString().split('T')[0]
      } catch {
          return dateString
      }
  }

  return (
    <DashboardLayout>
        <div className="container mx-auto p-8 max-w-4xl">
        <div className="markdown-content">
            <h1>{pageMetadata.title}</h1>

            {/* Document Properties Table */}
            <div className="doc-metadata-container">
                <table className="doc-metadata-table">
                    <tbody>
                        {pageMetadata.author && (
                            <tr>
                                <th>Author</th>
                               <td>{pageMetadata.author}</td>
                            </tr>
                        )}
                         <tr>
                            <th>Last Updated</th>
                            <td>{formatDate(pageMetadata.lastUpdated)}</td>
                        </tr>
                        {pageMetadata.effectiveDate && (
                            <tr>
                                <th>Effective Date</th>
                                <td>{formatDate(pageMetadata.effectiveDate)}</td>
                            </tr>
                        )}
                    </tbody>
                </table>
            </div>

            <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>
        </div>
        </div>
    </DashboardLayout>
  )
}
