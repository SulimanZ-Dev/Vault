export type LocalNotification = {
  id: number
  severity: 'info' | 'warning' | 'urgent'
  category: string
  title: string
  body: string
  document_id: number | null
  due_date: string | null
  read_at: string | null
  created_at: string
}

type Props = {
  notifications: LocalNotification[]
  onRefresh: () => void
  onUpdate: (id: number, dismiss: boolean) => void
  onOpenDocument: (documentId: number) => void
}

export function NotificationsView({ notifications, onRefresh, onUpdate, onOpenDocument }: Props) {
  return (
    <div className="test-center-layout">
      <section className="work-item form-stack">
        <strong>Notiser stannar på datorn</strong>
        <p>Vault skapar lokala notiser för utgående giltighet, granskningskö och misslyckade jobb. Ingen data eller notis skickas till en extern tjänst.</p>
        <button className="secondary-action" onClick={onRefresh} type="button">Uppdatera notiser</button>
      </section>
      <section className="work-panel-grid">
        {notifications.map(item => <article className={`work-item notification-${item.severity}`} key={item.id}>
          <strong>{item.title}</strong><span>{item.body}</span>
          <small>{item.category}{item.due_date ? ` · senast ${item.due_date}` : ''}</small>
          <div className="claim-actions">
            {!item.read_at ? <button className="mini-action" onClick={() => onUpdate(item.id, false)} type="button">Markera läst</button> : null}
            <button className="mini-action" onClick={() => onUpdate(item.id, true)} type="button">Avfärda</button>
            {item.document_id ? <button className="mini-action" onClick={() => onOpenDocument(item.document_id!)} type="button">Öppna dokument</button> : null}
          </div>
        </article>)}
        {!notifications.length ? <div className="empty-state">Inga aktiva notiser.</div> : null}
      </section>
    </div>
  )
}
