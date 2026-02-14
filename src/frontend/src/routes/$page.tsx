import { createFileRoute, notFound } from '@tanstack/react-router'

export const Route = createFileRoute('/$page')({
  loader: async ({ params }) => {
    const page = params.page
    const metadata = __MARKDOWN_METADATA__

    if (!metadata[page]) {
      throw notFound()
    }

    return {
      metadata: metadata[page],
      page
    }
  }
})
