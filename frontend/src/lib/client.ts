import type { paths } from './types'
import createClient from 'openapi-fetch'

export const client = createClient<paths>({
  baseUrl: '/api',
})

