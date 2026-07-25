import { invoke } from '@tauri-apps/api/core'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import * as pdfjs from 'pdfjs-dist/legacy/build/pdf.mjs'
import pdfWorkerUrl from 'pdfjs-dist/legacy/build/pdf.worker.min.mjs?url'
import type { PDFDocumentProxy, PDFPageProxy, TextItem } from 'pdfjs-dist/types/src/display/api'

pdfjs.GlobalWorkerOptions.workerSrc = pdfWorkerUrl

type ViewerContent = {
  document_id: number
  version_id: number
  original_name: string
  mime_type: string
  sha256: string
  size_bytes: number
  page_count_hint: number
  content_base64: string
}

type TextSpan = {
  text: string
  left: number
  top: number
  width: number
  height: number
  angle: number
}

type Props = {
  documentId: number
  title: string
  pin: string
  initialPage?: number
  onClose: () => void
  onPageChange?: (page: number) => void
  initialSearch?: string
  sourceClaimId?: number | null
  sourceRule?: string
}

type ViewerAnnotation = {
  id: number
  document_id: number
  document_version_id: number | null
  page_no: number
  annotation_type: string
  selected_text: string
  body: string
  color: string
  x: number | null
  y: number | null
  width: number | null
  height: number | null
  claim_id: number | null
  rule_result: string
}

function decodeBase64(value: string) {
  const binary = window.atob(value)
  const bytes = new Uint8Array(binary.length)
  for (let index = 0; index < binary.length; index += 1) bytes[index] = binary.charCodeAt(index)
  return bytes
}

function Thumbnail({ pdf, pageNumber, active, onSelect }: { pdf: PDFDocumentProxy; pageNumber: number; active: boolean; onSelect: () => void }) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  useEffect(() => {
    let cancelled = false
    let task: ReturnType<PDFPageProxy['render']> | null = null
    void pdf.getPage(pageNumber).then((page) => {
      if (cancelled || !canvasRef.current) return
      const base = page.getViewport({ scale: 1 })
      const viewport = page.getViewport({ scale: 132 / base.width })
      const canvas = canvasRef.current
      canvas.width = Math.ceil(viewport.width)
      canvas.height = Math.ceil(viewport.height)
      task = page.render({ canvas, canvasContext: canvas.getContext('2d')!, viewport })
      return task.promise
    }).catch(() => undefined)
    return () => { cancelled = true; task?.cancel() }
  }, [pageNumber, pdf])
  return <button aria-current={active ? 'page' : undefined} className={active ? 'pdf-thumbnail active' : 'pdf-thumbnail'} onClick={onSelect} type="button"><canvas ref={canvasRef}/><span>Sida {pageNumber}</span></button>
}

export function DocumentViewer({ documentId, title, pin, initialPage = 1, onClose, onPageChange, initialSearch = '', sourceClaimId = null, sourceRule = '' }: Props) {
  const [content, setContent] = useState<ViewerContent | null>(null)
  const [pdf, setPdf] = useState<PDFDocumentProxy | null>(null)
  const [page, setPage] = useState(Math.max(1, initialPage))
  const [pageInput, setPageInput] = useState(String(Math.max(1, initialPage)))
  const [zoom, setZoom] = useState(1)
  const [rotation, setRotation] = useState(0)
  const [fitMode, setFitMode] = useState<'custom' | 'width' | 'page'>('width')
  const [query, setQuery] = useState(initialSearch)
  const [annotations, setAnnotations] = useState<ViewerAnnotation[]>([])
  const [drawing, setDrawing] = useState(false)
  const [drawStart, setDrawStart] = useState<{ x: number; y: number } | null>(null)
  const [draftRect, setDraftRect] = useState<{ x: number; y: number; width: number; height: number } | null>(null)
  const [textSpans, setTextSpans] = useState<TextSpan[]>([])
  const [status, setStatus] = useState('Läser originalfil…')
  const [error, setError] = useState<string | null>(null)
  const [containerWidth, setContainerWidth] = useState(900)
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const stageRef = useRef<HTMLDivElement>(null)
  const pageShellRef = useRef<HTMLDivElement>(null)
  const renderTaskRef = useRef<ReturnType<PDFPageProxy['render']> | null>(null)
  const textCache = useRef(new Map<number, string>())

  useEffect(() => {
    const observer = new ResizeObserver((entries) => setContainerWidth(Math.max(320, entries[0]?.contentRect.width ?? 900)))
    if (stageRef.current) observer.observe(stageRef.current)
    return () => observer.disconnect()
  }, [])

  useEffect(() => {
    let cancelled = false
    let loadedPdf: PDFDocumentProxy | null = null
    const cache = textCache.current
    setError(null)
    setStatus('Läser originalfil…')
    invoke<ViewerContent>('get_document_viewer_content', { documentId, pin: pin || null })
      .then((value) => {
        if (cancelled) return
        setContent(value)
        void invoke<ViewerAnnotation[]>('list_document_annotations', { documentId }).then(setAnnotations)
        const saved = localStorage.getItem(`vault.viewer.${documentId}.${value.sha256}`)
        if (saved) {
          try {
            const state = JSON.parse(saved) as { page?: number; zoom?: number; rotation?: number; fitMode?: 'custom' | 'width' | 'page' }
            setPage(Math.max(1, state.page ?? initialPage))
            setZoom(Math.min(4, Math.max(.25, state.zoom ?? 1)))
            setRotation(state.rotation ?? 0)
            setFitMode(state.fitMode ?? 'width')
          } catch { /* Ignorera en skadad lokal visningspreferens. */ }
        }
        if (value.mime_type === 'application/pdf') {
          const task = pdfjs.getDocument({ data: decodeBase64(value.content_base64), isEvalSupported: false })
          return task.promise.then((loaded) => { loadedPdf = loaded; if (!cancelled) setPdf(loaded) })
        }
        setStatus('Originalet visas lokalt')
      })
      .catch((reason) => setError(String(reason)))
    return () => { cancelled = true; setPdf(null); cache.clear(); void loadedPdf?.destroy() }
  }, [documentId, initialPage, pin])

  const pageCount = pdf?.numPages ?? (content?.mime_type.startsWith('image/') ? 1 : Math.max(1, content?.page_count_hint ?? 1))
  useEffect(() => {
    const next = Math.min(pageCount, Math.max(1, page))
    if (next !== page) setPage(next)
    setPageInput(String(next))
    onPageChange?.(next)
  }, [onPageChange, page, pageCount])

  useEffect(() => {
    if (!content) return
    localStorage.setItem(`vault.viewer.${documentId}.${content.sha256}`, JSON.stringify({ page, zoom, rotation, fitMode }))
  }, [content, documentId, fitMode, page, rotation, zoom])

  useEffect(() => {
    if (!pdf || !canvasRef.current) return
    let cancelled = false
    renderTaskRef.current?.cancel()
    setStatus(`Renderar sida ${page}…`)
    void pdf.getPage(page).then(async (pdfPage) => {
      if (cancelled || !canvasRef.current) return
      const base = pdfPage.getViewport({ scale: 1, rotation })
      const availableWidth = Math.max(280, containerWidth - 48)
      const availableHeight = Math.max(320, window.innerHeight - 190)
      const scale = fitMode === 'width' ? availableWidth / base.width : fitMode === 'page' ? Math.min(availableWidth / base.width, availableHeight / base.height) : zoom
      const viewport = pdfPage.getViewport({ scale, rotation })
      const canvas = canvasRef.current
      const ratio = Math.min(window.devicePixelRatio || 1, 2)
      canvas.width = Math.ceil(viewport.width * ratio)
      canvas.height = Math.ceil(viewport.height * ratio)
      canvas.style.width = `${Math.ceil(viewport.width)}px`
      canvas.style.height = `${Math.ceil(viewport.height)}px`
      const context = canvas.getContext('2d')!
      const task = pdfPage.render({ canvas, canvasContext: context, viewport, transform: ratio === 1 ? undefined : [ratio, 0, 0, ratio, 0, 0] })
      renderTaskRef.current = task
      await task.promise
      if (cancelled) return
      const text = await pdfPage.getTextContent()
      const spans = text.items.filter((item): item is TextItem => 'str' in item).map((item) => {
        const transform = pdfjs.Util.transform(viewport.transform, item.transform)
        const angle = Math.atan2(transform[1], transform[0])
        const height = Math.hypot(transform[2], transform[3])
        return { text: item.str, left: transform[4], top: transform[5] - height, width: Math.max(1, item.width * scale), height: Math.max(1, height), angle }
      })
      textCache.current.set(page, spans.map((span) => span.text).join(' '))
      setTextSpans(spans)
      setStatus(`Sida ${page} av ${pdf.numPages}`)
    }).catch((reason) => { if (!cancelled && String(reason?.name) !== 'RenderingCancelledException') setError(String(reason)) })
    return () => { cancelled = true; renderTaskRef.current?.cancel() }
  }, [containerWidth, fitMode, page, pdf, rotation, zoom])

  const goToPage = useCallback((value: number) => setPage(Math.min(pageCount, Math.max(1, value))), [pageCount])
  const findNext = useCallback(async (direction: 1 | -1) => {
    if (!pdf || !query.trim()) return
    const needle = query.toLocaleLowerCase('sv-SE')
    for (let offset = 0; offset < pdf.numPages; offset += 1) {
      const candidate = ((page - 1 + direction * (offset + 1) + pdf.numPages) % pdf.numPages) + 1
      let haystack = textCache.current.get(candidate)
      if (haystack == null) {
        const text = await (await pdf.getPage(candidate)).getTextContent()
        haystack = text.items.filter((item): item is TextItem => 'str' in item).map((item) => item.str).join(' ')
        textCache.current.set(candidate, haystack)
      }
      if (haystack.toLocaleLowerCase('sv-SE').includes(needle)) { goToPage(candidate); return }
    }
    setStatus(`Ingen träff för ”${query}”`)
  }, [goToPage, page, pdf, query])

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (['INPUT', 'TEXTAREA', 'SELECT'].includes((event.target as HTMLElement)?.tagName)) return
      if (event.key === 'Escape') onClose()
      else if (event.key === 'ArrowRight' || event.key === 'PageDown') goToPage(page + 1)
      else if (event.key === 'ArrowLeft' || event.key === 'PageUp') goToPage(page - 1)
      else if (event.key === 'Home') goToPage(1)
      else if (event.key === 'End') goToPage(pageCount)
      else if (event.key === '+' || event.key === '=') { setFitMode('custom'); setZoom((value) => Math.min(4, value + .1)) }
      else if (event.key === '-') { setFitMode('custom'); setZoom((value) => Math.max(.25, value - .1)) }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [goToPage, onClose, page, pageCount])

  const thumbnailPages = useMemo(() => {
    if (!pdf) return []
    const start = Math.max(1, page - 12)
    const end = Math.min(pdf.numPages, page + 12)
    return Array.from({ length: end - start + 1 }, (_, index) => start + index)
  }, [page, pdf])
  const imageUrl = content?.mime_type.startsWith('image/') ? `data:${content.mime_type};base64,${content.content_base64}` : null
  const normalizedQuery = query.toLocaleLowerCase('sv-SE')
  const pageAnnotations = annotations.filter((item) => item.page_no === page && item.document_version_id === content?.version_id && item.x != null && item.y != null && item.width != null && item.height != null)

  function pointerPosition(event: React.PointerEvent<HTMLDivElement>) {
    const bounds = pageShellRef.current?.getBoundingClientRect()
    if (!bounds) return null
    return { x: Math.min(1, Math.max(0, (event.clientX - bounds.left) / bounds.width)), y: Math.min(1, Math.max(0, (event.clientY - bounds.top) / bounds.height)) }
  }

  function beginArea(event: React.PointerEvent<HTMLDivElement>) {
    if (!drawing || rotation !== 0) return
    const position = pointerPosition(event)
    if (!position) return
    event.currentTarget.setPointerCapture(event.pointerId)
    setDrawStart(position)
    setDraftRect({ ...position, width: 0, height: 0 })
  }

  function moveArea(event: React.PointerEvent<HTMLDivElement>) {
    if (!drawStart) return
    const position = pointerPosition(event)
    if (!position) return
    setDraftRect({ x: Math.min(drawStart.x, position.x), y: Math.min(drawStart.y, position.y), width: Math.abs(position.x - drawStart.x), height: Math.abs(position.y - drawStart.y) })
  }

  async function finishArea(event: React.PointerEvent<HTMLDivElement>) {
    if (!drawStart || !content) return
    const position = pointerPosition(event)
    setDrawStart(null)
    if (!position) return
    const rectangle = { x: Math.min(drawStart.x, position.x), y: Math.min(drawStart.y, position.y), width: Math.abs(position.x - drawStart.x), height: Math.abs(position.y - drawStart.y) }
    setDraftRect(null)
    if (rectangle.width <= .005 || rectangle.height <= .005) return
    try {
      const selectedText = window.getSelection()?.toString().trim() ?? ''
      const saved = await invoke<ViewerAnnotation[]>('save_coordinate_annotation', { documentId, documentVersionId: content.version_id, pageNo: page, ...rectangle, selectedText, body: sourceClaimId ? 'Källområde för claim' : 'Visuell markering', color: '#ffd54a', claimId: sourceClaimId, ruleResult: sourceRule })
      setAnnotations(saved)
      setDrawing(false)
      setStatus('Markeringen sparades med normaliserade koordinater')
    } catch (reason) { setError(String(reason)) }
  }

  return <section aria-label={`Dokumentvisare: ${title}`} aria-modal="true" className="native-viewer" role="dialog">
    <header className="native-viewer-toolbar">
      <div><strong>{title}</strong><small>{content?.original_name ?? status}</small></div>
      <nav aria-label="Sidnavigation">
        <button disabled={page <= 1} onClick={() => goToPage(page - 1)} type="button">←</button>
        <label>Sida <input aria-label="Sidnummer" inputMode="numeric" value={pageInput} onChange={(event) => setPageInput(event.target.value)} onKeyDown={(event) => { if (event.key === 'Enter') goToPage(Number(pageInput)) }}/></label>
        <span>av {pageCount}</span>
        <button disabled={page >= pageCount} onClick={() => goToPage(page + 1)} type="button">→</button>
      </nav>
      <nav aria-label="Zoom och rotation">
        <button onClick={() => { setFitMode('custom'); setZoom((value) => Math.max(.25, value - .1)) }} type="button">−</button>
        <span>{Math.round(zoom * 100)}%</span>
        <button onClick={() => { setFitMode('custom'); setZoom((value) => Math.min(4, value + .1)) }} type="button">+</button>
        <button className={fitMode === 'width' ? 'active' : ''} onClick={() => setFitMode('width')} type="button">Bredd</button>
        <button className={fitMode === 'page' ? 'active' : ''} onClick={() => setFitMode('page')} type="button">Hela sidan</button>
        <button onClick={() => setRotation((value) => (value + 90) % 360)} type="button">Rotera</button>
        <button className={drawing ? 'active' : ''} disabled={!pdf} onClick={() => { if (rotation !== 0) { setStatus('Återställ rotation till 0° innan ett område markeras'); return } setDrawing((value) => !value) }} type="button">{drawing ? 'Avbryt markering' : 'Markera område'}</button>
        <button onClick={onClose} type="button">Stäng</button>
      </nav>
    </header>
    <div className="native-viewer-search"><input aria-label="Sök i dokumentet" placeholder="Sök i PDF-text eller OCR…" value={query} onChange={(event) => setQuery(event.target.value)} onKeyDown={(event) => { if (event.key === 'Enter') void findNext(1) }}/><button disabled={!query.trim()} onClick={() => void findNext(-1)} type="button">Föregående träff</button><button disabled={!query.trim()} onClick={() => void findNext(1)} type="button">Nästa träff</button><span aria-live="polite">{status}</span></div>
    {error ? <div className="native-viewer-error" role="alert"><strong>Dokumentet kunde inte visas</strong><span>{error}</span><button onClick={onClose}>Stäng</button></div> : null}
    <div className="native-viewer-body">
      {pdf ? <aside aria-label="Sidminiatyrer" className="pdf-thumbnails"><div className="thumbnail-spacer" style={{ height: `${Math.max(0, thumbnailPages[0] - 1) * 190}px` }}/>{thumbnailPages.map((number) => <Thumbnail active={number === page} key={number} onSelect={() => goToPage(number)} pageNumber={number} pdf={pdf}/>)}</aside> : null}
      <div className="native-viewer-stage" ref={stageRef}>
        {!content && !error ? <div className="viewer-loading">Läser och verifierar originalfil…</div> : null}
        {pdf ? <div className="pdf-page-shell" ref={pageShellRef}><canvas ref={canvasRef}/><div aria-label="Kopierbart PDF-textlager" className="pdf-text-layer">{textSpans.map((span, index) => <span className={normalizedQuery && span.text.toLocaleLowerCase('sv-SE').includes(normalizedQuery) ? 'search-hit' : ''} key={`${index}-${span.left}`} style={{ height: span.height, left: span.left, lineHeight: `${span.height}px`, top: span.top, transform: `rotate(${span.angle}rad)`, width: span.width }}>{span.text}</span>)}</div><div className={drawing ? 'pdf-annotation-layer drawing' : 'pdf-annotation-layer'} onPointerDown={beginArea} onPointerMove={moveArea} onPointerUp={(event) => void finishArea(event)}>{pageAnnotations.map((item) => <button aria-label={`${item.claim_id ? 'Claim-källa' : 'Markering'}: ${item.body}`} className={item.claim_id ? 'page-annotation claim-source' : 'page-annotation'} key={item.id} style={{ backgroundColor: `${item.color}66`, borderColor: item.color, height: `${item.height! * 100}%`, left: `${item.x! * 100}%`, top: `${item.y! * 100}%`, width: `${item.width! * 100}%` }} title={`${item.body}${item.selected_text ? `: ${item.selected_text}` : ''}`} type="button"/>) }{draftRect ? <span className="page-annotation draft" style={{ height: `${draftRect.height * 100}%`, left: `${draftRect.x * 100}%`, top: `${draftRect.y * 100}%`, width: `${draftRect.width * 100}%` }}/> : null}</div></div> : null}
        {imageUrl ? <img alt={title} className="native-image-document" src={imageUrl} style={{ transform: `rotate(${rotation}deg)`, width: fitMode === 'width' ? '100%' : fitMode === 'page' ? 'auto' : `${zoom * 100}%`, maxHeight: fitMode === 'page' ? 'calc(100vh - 190px)' : undefined }}/>:null}
        {content && !pdf && !imageUrl && !error ? <div className="viewer-unsupported"><strong>Intern rendering stöds inte för {content.mime_type}</strong><span>Använd den extraherade textvyn eller öppna originalet i standardappen.</span></div> : null}
      </div>
    </div>
  </section>
}
