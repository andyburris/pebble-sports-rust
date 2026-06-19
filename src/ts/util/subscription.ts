const subscriptions = new Map<number, ReturnType<typeof setInterval>>()

export function unsubscribe(windowId: number) {
    const interval = subscriptions.get(windowId)
    if (interval) {
        clearInterval(interval)
        subscriptions.delete(windowId)
        console.log(`Unsubscribed window ${windowId}`)
    }
}

export function subscribe(windowId: number, intervalMs: number, onTick: (initial: boolean) => void) {
    unsubscribe(windowId)
    onTick(true)
    const interval = setInterval(() => onTick(false), intervalMs)
    subscriptions.set(windowId, interval)
    console.log(`Subscribed window ${windowId}`)
}
