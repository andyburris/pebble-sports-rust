import { AppMessageData } from ".."


export async function sendItemList(
    items: AppMessageData[],
    windowId: number,
): Promise<void> {
    if(items.length === 0) {
        await sendAppMessage({
            taconite_WindowId: windowId,
            taconite_ItemIndex: 0,
            taconite_ItemTotal: 0,
        })
        return
    }
    
    for (let i = 0; i < items.length; i++) {
        await sendAppMessage({
            taconite_WindowId: windowId,
            taconite_ItemIndex: i,
            taconite_ItemTotal: items.length,
            ...items[i],
        }).catch(e => console.error(e))
    }
}

export async function sendAdvancedList<T>(items: T[], windowId: number, sendEach: (item: T, send: (message: AppMessageData) => Promise<void>) => Promise<void>) {
    if(items.length === 0) {
        await sendAppMessage({
            taconite_WindowId: windowId,
            taconite_ItemIndex: 0,
            taconite_ItemTotal: 0,
        })
        return
    }
    for (let i = 0; i < items.length; i++) {
        const item = items[i]
        const sendFunction = async (message: AppMessageData) => {
            await sendAppMessage({
                taconite_WindowId: windowId,
                taconite_ItemIndex: i,
                taconite_ItemTotal: items.length,
                ...message,
            }).catch(e => console.error(e))
        }
        await sendEach(item, sendFunction)
    }

}

export async function sendAppMessage(data: AppMessageData) {
    return await PebbleTS.sendAppMessage(data).catch(e => console.error(e))
}