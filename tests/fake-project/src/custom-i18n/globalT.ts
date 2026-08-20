import { t } from '@custom/i18n'

export const globalT = t('GLOBAL_T')

export const globalValidationRules = {
  message: t('GLOBAL_T_REQUIRED', 'This field is required'),
  validator(value: string) {
    if (!value) return t('GLOBAL_T_REQUIRED', 'This field is required')

    return validate({
      format: t('GLOBAL_T_FORMAT', 'Enter the expected format'),
      nested: {
        length: t('GLOBAL_T_LENGTH', 'Enter no more than 255 characters'),
      },
    })
  },
  label: t('GLOBAL_T_LABEL', 'Field label'),
}

function validate(messages: Record<string, unknown>) {
  return messages
}
