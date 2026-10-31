import { createContext, useContext } from 'react'
import type { FireLightingModel } from './fireLighting'

export const FireLightingContext = createContext<FireLightingModel | null>(null)
export const useFireLighting = (): FireLightingModel | null => useContext(FireLightingContext)
