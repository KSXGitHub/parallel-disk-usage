import Ajv from 'ajv'
import addFormats from 'ajv-formats'

export const AJV_FORMATS = [
  'date-time',
  'time',
  'date',
  'email',
  'hostname',
  'ipv4',
  'ipv6',
  'uri',
  'uri-reference',
  'uuid',
  'uri-template',
  'json-pointer',
  'relative-json-pointer',
  'regex',
] as const

export const createAjv = () => addFormats(new Ajv(), [...AJV_FORMATS]).addKeyword('kind').addKeyword('modifier')
export default createAjv
